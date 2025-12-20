use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use pagi_common::{publish_event, EventEnvelope, EventType, Playbook};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::net::SocketAddr;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use uuid::Uuid;

#[derive(Clone)]
struct AppState {
    working_memory_url: String,
    http: reqwest::Client,
    ethics: EthicsLayer,
    principles: PrinciplesLayer,
}

#[derive(Clone, Debug, Default)]
struct EthicsLayer {
    enabled: bool,
    constitution: Option<String>,
    red_lines: Vec<String>,
}

impl EthicsLayer {
    fn from_env() -> Self {
        let enabled = std::env::var("ETHICS_ALIGNMENT_CHECK")
            .unwrap_or_else(|_| "false".to_string())
            .to_lowercase()
            == "true";

        let constitution = std::env::var("ETHICS_CONSTITUTION").ok();
        let red_lines = std::env::var("ETHICS_RED_LINES")
            .ok()
            .map(|s| split_list(&s))
            .unwrap_or_default();

        Self {
            enabled,
            constitution,
            red_lines,
        }
    }

    fn render(&self) -> Option<String> {
        if !self.enabled {
            return None;
        }
        let mut s = String::new();
        if let Some(c) = &self.constitution {
            s.push_str("# Ethics / Constitution\n");
            s.push_str(c);
            s.push('\n');
        }
        if !self.red_lines.is_empty() {
            s.push_str("\n# Red Lines\n");
            for rl in &self.red_lines {
                s.push_str(&format!("- {}\n", rl));
            }
        }
        Some(s)
    }
}

fn split_list(raw: &str) -> Vec<String> {
    raw.split(|c| c == ',' || c == '\n' || c == ';')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect()
}

#[derive(Clone, Debug, Default)]
struct PrinciplesLayer {
    core_values: Vec<String>,
    checkpoints: Vec<String>,
}

impl PrinciplesLayer {
    fn from_env() -> Self {
        let core_values = std::env::var("AI_PRINCIPLES_CORE_VALUES")
            .ok()
            .map(|s| split_list(&s))
            .unwrap_or_default();

        let checkpoints = std::env::var("AI_PRINCIPLES_ALIGNMENT_CHECKPOINTS")
            .ok()
            .map(|s| split_list(&s))
            .unwrap_or_default();

        Self { core_values, checkpoints }
    }

    fn render(&self, playbook: Option<&Playbook>) -> Option<String> {
        // Prefer env-gated values; fall back to playbook if provided.
        let (values, checkpoints) = if !self.core_values.is_empty() || !self.checkpoints.is_empty() {
            (&self.core_values, &self.checkpoints)
        } else if let Some(pb) = playbook.and_then(|p| p.ai_principles.as_ref()) {
            (&pb.core_values, &pb.alignment_checkpoints)
        } else {
            return None;
        };

        let mut s = String::new();
        if !values.is_empty() {
            s.push_str("# AI Principles\n");
            for v in values {
                s.push_str(&format!("- {}\n", v));
            }
        }
        if !checkpoints.is_empty() {
            s.push_str("\n# Alignment Checkpoints\n");
            for c in checkpoints {
                s.push_str(&format!("- {}\n", c));
            }
        }
        Some(s)
    }
}

#[derive(Debug, Deserialize)]
struct BuildRequest {
    pub twin_id: Uuid,
    #[serde(alias = "goal")]
    pub query: String,

    #[serde(default)]
    pub playbook: Option<Playbook>,
}

#[derive(Debug, Serialize)]
struct BuildResponse {
    pub twin_id: Uuid,
    pub context: String,
    pub sources: Vec<String>,
}

#[tokio::main]
async fn main() {
    pagi_http::tracing::init("pagi-context-builder");

    let state = AppState {
        working_memory_url: std::env::var("WORKING_MEMORY_URL")
            .unwrap_or_else(|_| "http://127.0.0.1:8003".to_string()),
        http: reqwest::Client::new(),
        ethics: EthicsLayer::from_env(),
        principles: PrinciplesLayer::from_env(),
    };

    let app = Router::new()
        .route("/healthz", get(healthz))
        .route("/build", post(build_context))
        .with_state(state)
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive());

    let addr: SocketAddr = pagi_http::config::bind_addr(([0, 0, 0, 0], 8004).into());
    tracing::info!(%addr, "listening");
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn healthz() -> (StatusCode, &'static str) {
    (StatusCode::OK, "ok")
}

async fn build_context(State(state): State<AppState>, Json(req): Json<BuildRequest>) -> Result<Json<BuildResponse>, (StatusCode, String)> {
    let mem_endpoint = format!(
        "{}/memory/{}",
        state.working_memory_url.trim_end_matches('/'),
        req.twin_id
    );
    let mem: Vec<serde_json::Value> = state
        .http
        .get(mem_endpoint)
        .send()
        .await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?
        .json()
        .await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;

    // Base memory layer.
    let mut memory_layer = String::new();
    memory_layer.push_str("# Working Memory\n");
    for item in &mem {
        let role = item.get("role").and_then(|v| v.as_str()).unwrap_or("unknown");
        let content = item
            .get("content")
            .and_then(|v| v.as_str())
            .unwrap_or_default();
        memory_layer.push_str(&format!("- {}: {}\n", role, content));
    }

    // If a playbook with ACE config is provided, assemble context using layers + priority.
    let context = if let Some(playbook) = &req.playbook {
        if let Some(ce) = &playbook.context_engineering {
            let mut layers: std::collections::HashMap<&str, String> = std::collections::HashMap::new();

            let system = if !ce.layers.system.trim().is_empty() {
                ce.layers.system.clone()
            } else {
                playbook.system_prompt().to_string()
            };
            layers.insert("system", system);
            layers.insert("reflection", ce.layers.reflection.clone());
            layers.insert("tools", ce.layers.tools.clone());
            layers.insert("memory", if !ce.layers.memory.trim().is_empty() { ce.layers.memory.clone() } else { memory_layer.clone() });

            let goal_layer = if !ce.layers.goal.trim().is_empty() {
                ce.layers.goal.replace("{{goal}}", &req.query)
            } else {
                format!("Current user goal: {}", req.query)
            };
            layers.insert("goal", goal_layer);

            if let Some(ethics) = state.ethics.render() {
                layers.insert("ethics", ethics);
            }

            if let Some(principles) = state.principles.render(Some(playbook)) {
                layers.insert("ai_principles", principles);
            }

            let priority = if ce.order.priority.is_empty() {
                vec!["system", "ethics", "ai_principles", "reflection", "tools", "memory", "goal"]
            } else {
                ce.order.priority.iter().map(|s| s.as_str()).collect()
            };

            let mut out = String::new();
            for key in priority {
                if let Some(val) = layers.get(key) {
                    if val.trim().is_empty() {
                        continue;
                    }
                    out.push_str(&format!("# {}\n{}\n\n", key, val.trim()));
                }
            }

            out
        } else {
            // No ACE config: preserve legacy behavior.
            format!("{}\n\n# Query\n{}", memory_layer, req.query)
        }
    } else {
        // No playbook: preserve legacy behavior.
        format!("{}\n\n# Query\n{}", memory_layer, req.query)
    };

    let resp = BuildResponse {
        twin_id: req.twin_id,
        context,
        sources: vec!["working_memory".to_string()],
    };

    let mut ev = EventEnvelope::new(EventType::ContextBuilt, json!({"twin_id": req.twin_id}));
    ev.twin_id = Some(req.twin_id);
    ev.source = Some("pagi-context-builder".to_string());
    let _ = publish_event(ev).await;

    Ok(Json(resp))
}
