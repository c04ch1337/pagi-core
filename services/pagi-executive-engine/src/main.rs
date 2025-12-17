use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use pagi_common::{publish_event, CoreEvent, EventEnvelope, EventType};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::net::SocketAddr;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use uuid::Uuid;

#[derive(Clone)]
struct AppState {
    context_builder_url: String,
    inference_gateway_url: String,
    emotion_state_url: String,
    sensor_actuator_url: String,
    http: reqwest::Client,
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

#[tokio::main]
async fn main() {
    pagi_http::tracing::init("pagi-executive-engine");

    let state = AppState {
        context_builder_url: std::env::var("CONTEXT_BUILDER_URL").unwrap_or_else(|_| "http://127.0.0.1:8004".to_string()),
        inference_gateway_url: std::env::var("INFERENCE_GATEWAY_URL").unwrap_or_else(|_| "http://127.0.0.1:8005".to_string()),
        emotion_state_url: std::env::var("EMOTION_STATE_URL").unwrap_or_else(|_| "http://127.0.0.1:8007".to_string()),
        sensor_actuator_url: std::env::var("SENSOR_ACTUATOR_URL").unwrap_or_else(|_| "http://127.0.0.1:8008".to_string()),
        http: reqwest::Client::new(),
    };

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
) -> Result<Json<InteractResponse>, (StatusCode, String)> {
    // 1) Publish GoalReceived
    let mut goal_ev = EventEnvelope::new_core(twin_id, CoreEvent::GoalReceived { goal: req.goal.clone() });
    goal_ev.source = Some("pagi-executive-engine".to_string());
    let _ = publish_event(goal_ev).await;

    // 2) Build context
    let context_url = format!("{}/build", state.context_builder_url.trim_end_matches('/'));
    let ctx: ContextBuildResponse = state
        .http
        .post(context_url)
        .json(&json!({"twin_id": twin_id, "goal": req.goal}))
        .send()
        .await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?
        .error_for_status()
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?
        .json()
        .await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;

    // 3) Inference
    let infer_url = format!("{}/infer", state.inference_gateway_url.trim_end_matches('/'));
    let inf: InferenceResponse = state
        .http
        .post(infer_url)
        .json(&json!({"twin_id": twin_id, "input": "generate plan", "context": ctx.context}))
        .send()
        .await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?
        .error_for_status()
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?
        .json()
        .await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;

    // 4) Emotion state (optional)
    let emotion_url = format!("{}/emotion/{}", state.emotion_state_url.trim_end_matches('/'), twin_id);
    let emotion: EmotionState = state
        .http
        .get(emotion_url)
        .send()
        .await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?
        .error_for_status()
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?
        .json()
        .await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;

    // 5) Generate plan (stub)
    let plan = format!("Plan: {} | mood={} stress={:?}", inf.output, emotion.mood, emotion.stress);

    // 6) Publish PlanGenerated
    let mut plan_ev = EventEnvelope::new_core(twin_id, CoreEvent::PlanGenerated { plan: plan.clone() });
    plan_ev.source = Some("pagi-executive-engine".to_string());
    let _ = publish_event(plan_ev).await;

    // 7) Send to SensorActuator (still a no-op executor)
    let act_url = format!("{}/act", state.sensor_actuator_url.trim_end_matches('/'));
    state
        .http
        .post(act_url)
        .json(&json!({"tool": "execute_plan", "args": {"twin_id": twin_id, "plan": plan}}))
        .send()
        .await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;

    Ok(Json(InteractResponse {
        status: "plan_executed".to_string(),
        output: plan,
    }))
}
