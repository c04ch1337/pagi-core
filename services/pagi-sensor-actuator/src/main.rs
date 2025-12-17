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

#[derive(Clone)]
struct AppState {
}

#[derive(Debug, Deserialize)]
struct ActionRequest {
    pub tool: String,
    #[serde(default)]
    pub args: serde_json::Value,
}

#[derive(Debug, Serialize)]
struct ActionResponse {
    pub accepted: bool,
    pub message: String,
}

#[tokio::main]
async fn main() {
    pagi_http::tracing::init("pagi-sensor-actuator");

    let state = AppState {
    };

    let app = Router::new()
        .route("/healthz", get(healthz))
        .route("/act", post(act))
        .with_state(state)
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive());

    let addr: SocketAddr = pagi_http::config::bind_addr(([0, 0, 0, 0], 8008).into());
    tracing::info!(%addr, "listening");
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn healthz() -> (StatusCode, &'static str) {
    (StatusCode::OK, "ok")
}

async fn act(State(_state): State<AppState>, Json(req): Json<ActionRequest>) -> (StatusCode, Json<ActionResponse>) {
    let mut ev = EventEnvelope::new(EventType::ActionRequested, json!({"tool": req.tool, "args": req.args}));
    ev.source = Some("pagi-sensor-actuator".to_string());
    let _ = publish_event(ev).await;

    // Intentionally minimal: actions are not executed in MVP.
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(ActionResponse {
            accepted: false,
            message: "MVP does not execute external actions".to_string(),
        }),
    )
}
