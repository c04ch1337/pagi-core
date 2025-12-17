use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use pagi_common::{publish_event, EventEnvelope, EventType};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::net::SocketAddr;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use uuid::Uuid;

#[derive(Clone)]
struct AppState {
}

#[derive(Debug, Deserialize)]
struct InferRequest {
    pub twin_id: Uuid,
    pub input: String,
    #[serde(default)]
    pub context: Option<String>,
}

#[derive(Debug, Serialize)]
struct InferResponse {
    pub twin_id: Uuid,
    pub model: String,
    pub output: String,
}

#[tokio::main]
async fn main() {
    pagi_http::tracing::init("pagi-inference-gateway");

    let state = AppState {
    };

    let app = Router::new()
        .route("/healthz", get(healthz))
        .route("/infer", post(infer))
        .with_state(state)
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive());

    let addr: SocketAddr = pagi_http::config::bind_addr(([0, 0, 0, 0], 8005).into());
    tracing::info!(%addr, "listening");
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn healthz() -> (StatusCode, &'static str) {
    (StatusCode::OK, "ok")
}

async fn infer(State(_state): State<AppState>, Json(req): Json<InferRequest>) -> Result<Json<InferResponse>, (StatusCode, String)> {
    let mut ev = EventEnvelope::new(
        EventType::InferenceRequested,
        json!({"twin_id": req.twin_id, "has_context": req.context.is_some()}),
    );
    ev.twin_id = Some(req.twin_id);
    ev.source = Some("pagi-inference-gateway".to_string());
    let _ = publish_event(ev).await;

    // MVP mock model adapter: returns a deterministic response.
    let output = if let Some(ctx) = &req.context {
        format!("[mock-model] Context:\n{}\n\nInput:\n{}", ctx, req.input)
    } else {
        format!("[mock-model] Input:\n{}", req.input)
    };

    let mut ev = EventEnvelope::new(
        EventType::InferenceCompleted,
        json!({"twin_id": req.twin_id, "output_len": output.len()}),
    );
    ev.twin_id = Some(req.twin_id);
    ev.source = Some("pagi-inference-gateway".to_string());
    let _ = publish_event(ev).await;

    Ok(Json(InferResponse {
        twin_id: req.twin_id,
        model: "mock".to_string(),
        output,
    }))
}
