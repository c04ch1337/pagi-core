use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use pagi_common::{EventEnvelope, EventType};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::net::SocketAddr;
use tower_http::{cors::CorsLayer, trace::TraceLayer};

#[derive(Clone)]
struct AppState {
    event_router_url: Option<String>,
    http: reqwest::Client,
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
        event_router_url: std::env::var("EVENT_ROUTER_URL").ok(),
        http: reqwest::Client::new(),
    };

    let app = Router::new()
        .route("/healthz", get(healthz))
        .route("/act", post(act))
        .with_state(state)
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive());

    let addr: SocketAddr = pagi_http::config::bind_addr(([0, 0, 0, 0], 7008).into());
    tracing::info!(%addr, "listening");
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn healthz() -> (StatusCode, &'static str) {
    (StatusCode::OK, "ok")
}

async fn act(State(state): State<AppState>, Json(req): Json<ActionRequest>) -> (StatusCode, Json<ActionResponse>) {
    publish_event(
        &state,
        EventEnvelope::new(EventType::ActionRequested, json!({"tool": req.tool, "args": req.args})),
    )
    .await;

    // Intentionally minimal: actions are not executed in MVP.
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(ActionResponse {
            accepted: false,
            message: "MVP does not execute external actions".to_string(),
        }),
    )
}

async fn publish_event(state: &AppState, mut ev: EventEnvelope) {
    let Some(url) = state.event_router_url.as_deref() else {
        return;
    };
    ev.source = Some("pagi-sensor-actuator".to_string());
    let endpoint = format!("{}/publish", url.trim_end_matches('/'));
    let res = state.http.post(endpoint).json(&ev).send().await;
    if let Err(err) = res {
        tracing::warn!(error = %err, "failed to publish event");
    }
}

