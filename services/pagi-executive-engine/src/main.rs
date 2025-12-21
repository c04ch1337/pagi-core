use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use pagi_common::{
    publish_event, CoreEvent, EventEnvelope, EventType, InstructionsField, Playbook, PlaybookInstructions,
    RefinementArtifact, TwinId,
};
use pagi_http::errors::PagiAxumError;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::net::SocketAddr;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use uuid::Uuid;
use std::time::Duration;

#[derive(Clone)]
struct AppState {
    context_builder_url: String,
    inference_gateway_url: String,
    emotion_state_url: String,
    sensor_actuator_url: String,
    external_gateway_url: String,
    http: reqwest::Client,
    ethics: EthicsPolicy,
}

#[derive(Debug, Clone)]
struct EthicsPolicy {
    alignment_check: bool,
    #[allow(dead_code)]
    constitution: Option<String>,
    harm_categories: Vec<String>,
    red_lines: Vec<String>,
    refusal_response: String,
}

impl EthicsPolicy {
    fn from_env() -> Self {
        let alignment_check = std::env::var("ETHICS_ALIGNMENT_CHECK")
            .unwrap_or_else(|_| "false".to_string())
            .to_lowercase()
            == "true";

        let constitution = std::env::var("ETHICS_CONSTITUTION").ok();

        let harm_categories = std::env::var("ETHICS_HARM_CATEGORIES")
            .ok()
            .map(|s| split_list(&s))
            .unwrap_or_default();

        let red_lines = std::env::var("ETHICS_RED_LINES")
            .ok()
            .map(|s| split_list(&s))
            .unwrap_or_else(|| {
                vec![
                    "weapons".to_string(),
                    "elections".to_string(),
                    "non-consensual surveillance".to_string(),
                ]
            });

        let refusal_response = std::env::var("ETHICS_REFUSAL_RESPONSE").unwrap_or_else(|_| {
            "I cannot assist with that request as it conflicts with my ethical guidelines.".to_string()
        });

        Self {
            alignment_check,
            constitution,
            harm_categories,
            red_lines,
            refusal_response,
        }
    }

    fn check_goal(&self, goal: &str) -> Result<(), String> {
        if !self.alignment_check {
            return Ok(());
        }

        let g = goal.to_lowercase();

        // MVP: keyword-based red-line screening.
        for rule in &self.red_lines {
            let needle = rule.to_lowercase();
            if !needle.is_empty() && g.contains(&needle) {
                return Err(self.refusal_response.clone());
            }
        }

        // Optional harm-category screening.
        for cat in &self.harm_categories {
            let needle = cat.to_lowercase();
            if !needle.is_empty() && g.contains(&needle) {
                return Err(self.refusal_response.clone());
            }
        }

        Ok(())
    }
}

fn split_list(raw: &str) -> Vec<String> {
    raw.split(|c| c == ',' || c == '\n' || c == ';')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect()
}

#[derive(Debug, Deserialize)]
struct PlanRequest {
    pub twin_id: Option<Uuid>,
    pub goal: String,
}

#[derive(Debug, Serialize)]
struct PlanResponse {
    pub steps: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct InteractRequest {
    pub goal: String,
}

#[derive(Debug, Serialize)]
struct InteractResponse {
    pub status: String,
    pub output: String,
}

#[derive(Debug, Deserialize)]
struct ContextBuildResponse {
    #[allow(dead_code)]
    pub twin_id: Uuid,
    pub context: String,
}

#[derive(Debug, Deserialize)]
struct InferenceResponse {
    pub output: String,
}

#[derive(Debug, Deserialize)]
struct EmotionState {
    pub mood: String,
    #[serde(default)]
    pub stress: Option<f32>,
}

#[derive(Debug, Deserialize)]
struct ToolSchema {
    pub name: String,
    #[allow(dead_code)]
    pub description: String,
    #[allow(dead_code)]
    pub plugin_url: String,
    #[allow(dead_code)]
    pub endpoint: String,
    #[allow(dead_code)]
    pub parameters: Value,
}

#[derive(Debug, Deserialize)]
struct ToolsResponse {
    pub tools: Vec<ToolSchema>,
}

#[derive(Debug, Serialize)]
struct ExecuteToolRequest {
    pub twin_id: TwinId,
    pub parameters: Value,
}

#[tokio::main]
async fn main() {
    pagi_http::tracing::init("pagi-executive-engine");

    let state = AppState {
        context_builder_url: std::env::var("CONTEXT_BUILDER_URL").unwrap_or_else(|_| "http://127.0.0.1:8004".to_string()),
        inference_gateway_url: std::env::var("INFERENCE_GATEWAY_URL").unwrap_or_else(|_| "http://127.0.0.1:8005".to_string()),
        emotion_state_url: std::env::var("EMOTION_STATE_URL").unwrap_or_else(|_| "http://127.0.0.1:8007".to_string()),
        sensor_actuator_url: std::env::var("SENSOR_ACTUATOR_URL").unwrap_or_else(|_| "http://127.0.0.1:8008".to_string()),
        external_gateway_url: std::env::var("EXTERNAL_GATEWAY_URL").unwrap_or_else(|_| "http://127.0.0.1:8010".to_string()),
        http: reqwest::Client::new(),
        ethics: EthicsPolicy::from_env(),
    };

    // Optional: self-update checks via ExternalGateway tool (implemented by the updater plugin).
    // This keeps the core immutable: the executive only *invokes* a tool; it never replaces itself.
    spawn_update_checker(state.clone());

    let app = Router::new()
        .route("/healthz", get(healthz))
        .route("/plan", post(plan))
        .route("/interact/:twin_id", post(interact))
        .with_state(state)
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive());

    let addr: SocketAddr = pagi_http::config::bind_addr(([0, 0, 0, 0], 8006).into());
    tracing::info!(%addr, "listening");
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

fn spawn_update_checker(state: AppState) {
    let enabled = std::env::var("AUTO_CHECK_UPDATES")
        .unwrap_or_else(|_| "false".to_string())
        .to_lowercase()
        == "true";
    if !enabled {
        return;
    }

    let interval_secs: u64 = std::env::var("UPDATE_CHECK_INTERVAL_SECS")
        .ok()
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(6 * 60 * 60);

    let auto_apply = std::env::var("AUTO_APPLY_UPDATES")
        .unwrap_or_else(|_| "false".to_string())
        .to_lowercase()
        == "true";

    tokio::spawn(async move {
        let twin_id = Uuid::nil();
        let mut ticker = tokio::time::interval(Duration::from_secs(interval_secs));

        loop {
            ticker.tick().await;

            let raw = match execute_tool_raw(&state, "check_update", twin_id, json!({})).await {
                Ok(r) => r,
                Err(err) => {
                    tracing::debug!(error = %err, "update check skipped/failed");
                    continue;
                }
            };

            let parsed = serde_json::from_str::<UpdaterCheckResponse>(&raw).ok();
            if let Some(p) = parsed {
                if p.update_available {
                    tracing::warn!(
                        current_version = %p.current_version,
                        latest_version = %p.latest_version,
                        release_url = %p.release_url,
                        "update available"
                    );

                    if auto_apply {
                        match execute_tool_raw(&state, "apply_update", twin_id, json!({"restart": true})).await {
                            Ok(r) => tracing::warn!("update applied (best-effort): {r}"),
                            Err(err) => tracing::warn!(error = %err, "update apply failed"),
                        }
                    }
                } else {
                    tracing::info!(current_version = %p.current_version, latest_version = %p.latest_version, "core is up-to-date");
                }
            } else {
                tracing::debug!("update check response was not json: {raw}");
            }
        }
    });
}

#[derive(Debug, Deserialize)]
struct UpdaterCheckResponse {
    current_version: String,
    latest_version: String,
    update_available: bool,
    release_url: String,
    #[allow(dead_code)]
    asset_name: Option<String>,
}

async fn healthz() -> (StatusCode, &'static str) {
    (StatusCode::OK, "ok")
}

async fn plan(State(_state): State<AppState>, Json(req): Json<PlanRequest>) -> Json<PlanResponse> {
    let mut steps = Vec::new();
    steps.push(format!("Clarify goal: {}", req.goal));
    steps.push("Collect relevant memory/context".to_string());
    steps.push("Call inference gateway with built context".to_string());
    steps.push("Evaluate result and update state".to_string());

    if let Some(twin_id) = req.twin_id {
        let mut ev = EventEnvelope::new(EventType::PlanCreated, json!({"twin_id": twin_id, "step_count": steps.len()}));
        ev.twin_id = Some(twin_id);
        ev.source = Some("pagi-executive-engine".to_string());
        let _ = publish_event(ev).await;
    }

    Json(PlanResponse { steps })
}

async fn interact(
    State(state): State<AppState>,
    Path(twin_id): Path<Uuid>,
    Json(req): Json<InteractRequest>,
) -> Result<Json<InteractResponse>, PagiAxumError> {
    // 1) Publish GoalReceived
    let mut goal_ev = EventEnvelope::new_core(twin_id, CoreEvent::GoalReceived { goal: req.goal.clone() });
    goal_ev.source = Some("pagi-executive-engine".to_string());
    let _ = publish_event(goal_ev).await;

    // 1b) Ethics gate (best-effort, env-configured). Refuse early.
    if let Err(refusal) = state.ethics.check_goal(&req.goal) {
        return Ok(Json(InteractResponse {
            status: "refused".to_string(),
            output: refusal,
        }));
    }

    // 1c) Pull latest Hive Playbook (best-effort) for context + refinement.
    let playbook = try_pull_latest_playbook(&state, twin_id).await.unwrap_or_default();

    // 2) Build context (include playbook so ContextBuilder can apply ACE layering).
    let context_url = format!("{}/build", state.context_builder_url.trim_end_matches('/'));
    let ctx: ContextBuildResponse = state
        .http
        .post(context_url)
        .json(&json!({"twin_id": twin_id, "goal": req.goal, "playbook": playbook}))
        .send()
        .await?
        .error_for_status()
        ?
        .json()
        .await?;

    // 3) Inference
    let playbook_context = if playbook.context_engineering.is_none() && !playbook.system_prompt().trim().is_empty() {
        format!("\n\n[HIVE_PLAYBOOK]\n{}", playbook.system_prompt())
    } else {
        "".to_string()
    };

    let full_context = format!("{}{}", ctx.context, playbook_context);
    let infer_url = format!("{}/infer", state.inference_gateway_url.trim_end_matches('/'));
    let inf: InferenceResponse = state
        .http
        .post(infer_url)
        .json(&json!({"twin_id": twin_id, "input": "generate plan", "context": full_context}))
        .send()
        .await?
        .error_for_status()
        ?
        .json()
        .await?;

    // 5) Emotion state (optional)
    let emotion_url = format!("{}/emotion/{}", state.emotion_state_url.trim_end_matches('/'), twin_id);
    let emotion: EmotionState = state
        .http
        .get(emotion_url)
        .send()
        .await?
        .error_for_status()
        ?
        .json()
        .await?;

    // 6) Discover available tools from ExternalGateway
    let tools_url = format!("{}/tools", state.external_gateway_url.trim_end_matches('/'));
    let tools_response: ToolsResponse = state
        .http
        .get(tools_url)
        .send()
        .await?
        .error_for_status()
        ?
        .json()
        .await?;

    // 7) Generate plan incorporating available tools
    let tool_names: Vec<String> = tools_response.tools.iter().map(|t| t.name.clone()).collect();
    let tools_summary = if tool_names.is_empty() {
        "No external tools available".to_string()
    } else {
        format!("Available tools: {}", tool_names.join(", "))
    };

    let plan = format!(
        "Plan: {} | mood={} stress={:?} | {}",
        inf.output, emotion.mood, emotion.stress, tools_summary
    );

    // 8) Publish PlanGenerated
    let mut plan_ev = EventEnvelope::new_core(twin_id, CoreEvent::PlanGenerated { plan: plan.clone() });
    plan_ev.source = Some("pagi-executive-engine".to_string());
    let _ = publish_event(plan_ev).await;

    // 9) Execute a sample tool if available (for demonstration)
    if let Some(sample_tool) = tools_response.tools.first() {
        let execute_url = format!(
            "{}/execute/{}",
            state.external_gateway_url.trim_end_matches('/'),
            sample_tool.name
        );

        let execute_payload = ExecuteToolRequest {
            twin_id: TwinId(twin_id),
            parameters: json!({"goal": req.goal}),
        };

        match state.http.post(execute_url).json(&execute_payload).send().await {
            Ok(resp) => {
                let status = resp.status();
                let body = resp.text().await.unwrap_or_default();
                if status.is_success() {
                    tracing::info!(twin_id = %twin_id, tool_name = %sample_tool.name, "Sample tool executed: {body}");
                } else {
                    tracing::warn!(twin_id = %twin_id, tool_name = %sample_tool.name, "Sample tool failed: {status} {body}");
                }
            }
            Err(err) => {
                tracing::warn!(twin_id = %twin_id, tool_name = %sample_tool.name, error = %err, "Sample tool call failed");
            }
        }
    }

    // 10) Send to SensorActuator
    let act_url = format!("{}/act", state.sensor_actuator_url.trim_end_matches('/'));
    state
        .http
        .post(act_url)
        .json(&json!({"tool": "execute_plan", "args": {"twin_id": twin_id, "plan": plan}}))
        .send()
        .await?;

    // 11) Self-improvement loop (best-effort): reflect and offer artifact to Hive sync plugin via ExternalGateway.
    let artifact = generate_refinement_artifact(twin_id, &req.goal, &plan, &playbook);
    tokio::spawn(async move {
        // Fire-and-forget; do not block user response.
        if let Err(err) = try_push_refinement_artifact(&state, twin_id, artifact).await {
            tracing::debug!(twin_id = %twin_id, error = %err, "refinement artifact push skipped/failed");
        }
    });

    Ok(Json(InteractResponse {
        status: "plan_executed".to_string(),
        output: plan,
    }))
}

fn generate_refinement_artifact(twin_id: Uuid, goal: &str, outcome: &str, base: &Playbook) -> RefinementArtifact {
    // MVP: deterministic critique + minimal playbook update.
    let critique = format!(
        "Reflection: Goal='{}' produced output_len={} (improve tool usage + evaluation loop).",
        goal,
        outcome.len()
    );

    let mut playbook = base.clone();
    playbook.version = playbook.version.saturating_add(1);
    if playbook.system_prompt().trim().is_empty() {
        playbook.instructions = InstructionsField::Structured(PlaybookInstructions {
            system_prompt: "You are a self-improving PAGI agent. Prioritize safety, accuracy, and efficiency. Reflect on every task: what succeeded, what failed, and propose refinements.".to_string(),
            reflection_rules: vec![
                "Analyze outcomes using success metrics.".to_string(),
                "Generalize edge cases to cross-domain improvements.".to_string(),
                "Emit a refinement artifact when improvement is meaningful.".to_string(),
            ],
            meta_learning: "Dynamically select sub-agents based on task type; optimize for minimal steps.".to_string(),
        });
    } else {
        // ACE-style curation: prefer appending structured rules over overwriting.
        if let InstructionsField::Structured(instr) = &mut playbook.instructions {
            let curated_rule = "After every task: evaluate against metrics + ethics, identify root cause, and propose a concrete playbook refinement.".to_string();
            if !instr.reflection_rules.iter().any(|r| r == &curated_rule) {
                instr.reflection_rules.push(curated_rule);
            }
        }
    }

    RefinementArtifact {
        twin_id: Some(TwinId(twin_id)),
        critique,
        updated_playbook: playbook,
    }
}

async fn execute_tool_raw(state: &AppState, tool_name: &str, twin_id: Uuid, parameters: Value) -> Result<String, String> {
    let url = format!(
        "{}/execute/{}",
        state.external_gateway_url.trim_end_matches('/'),
        tool_name
    );

    let payload = ExecuteToolRequest {
        twin_id: TwinId(twin_id),
        parameters,
    };

    let resp = state.http.post(url).json(&payload).send().await.map_err(|e| e.to_string())?;
    if resp.status().is_success() {
        resp.text().await.map_err(|e| e.to_string())
    } else {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        Err(format!("tool '{tool_name}' returned {status}: {body}"))
    }
}

async fn try_pull_latest_playbook(state: &AppState, twin_id: Uuid) -> Option<Playbook> {
    // Prefer Hive tool names; fall back to existing SwarmSync names.
    let tool_candidates = ["hive_pull", "swarm_sync_pull_latest_playbook"];
    for tool_name in tool_candidates {
        let Ok(raw) = execute_tool_raw(state, tool_name, twin_id, json!({})).await else {
            continue;
        };
        if let Ok(playbook) = serde_json::from_str::<Playbook>(&raw) {
            return Some(playbook);
        }
    }
    None
}

async fn try_push_refinement_artifact(
    state: &AppState,
    twin_id: Uuid,
    artifact: RefinementArtifact,
) -> Result<(), String> {
    // Execute via ExternalGateway (plugin provides the implementation).
    let parameters = serde_json::to_value(artifact).map_err(|e| e.to_string())?;
    let tool_candidates = ["hive_push", "swarm_sync_push_artifact"];

    for tool_name in tool_candidates {
        if execute_tool_raw(state, tool_name, twin_id, parameters.clone()).await.is_ok() {
            return Ok(());
        }
    }

    Err("no hive/swarm push tool available".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_refinement_artifact_increments_playbook_version() {
        let twin_id = Uuid::new_v4();
        let art = generate_refinement_artifact(twin_id, "do X", "result", &Playbook::default());
        assert_eq!(art.twin_id, Some(TwinId(twin_id)));
        assert_eq!(art.updated_playbook.version, 1);
        assert!(!art.critique.is_empty());
    }
}
