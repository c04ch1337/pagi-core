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
use uuid::Uuid;

#[derive(Clone)]
struct AppState {
    event_router_url: Option<String>,
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

#[tokio::main]
async fn main() {
    pagi_http::tracing::init("pagi-executive-engine");

    let state = AppState {
        event_router_url: std::env::var("EVENT_ROUTER_URL").ok(),
        http: reqwest::Client::new(),
    };

    let app = Router::new()
        .route("/healthz", get(healthz))
        .route("/plan", post(plan))
        .with_state(state)
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive());

    let addr: SocketAddr = pagi_http::config::bind_addr(([0, 0, 0, 0], 7006).into());
    tracing::info!(%addr, "listening");
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn healthz() -> (StatusCode, &'static str) {
    (StatusCode::OK, "ok")
}

async fn plan(State(state): State<AppState>, Json(req): Json<PlanRequest>) -> Json<PlanResponse> {
    let mut steps = Vec::new();
    steps.push(format!("Clarify goal: {}", req.goal));
    steps.push("Collect relevant memory/context".to_string());
    steps.push("Call inference gateway with built context".to_string());
    steps.push("Evaluate result and update state".to_string());

    if let Some(twin_id) = req.twin_id {
        publish_event(
            &state,
            EventEnvelope::new(EventType::PlanCreated, json!({"twin_id": twin_id, "step_count": steps.len()})),
        )
        .await;
    }

    Json(PlanResponse { steps })
}

async fn publish_event(state: &AppState, mut ev: EventEnvelope) {
    let Some(url) = state.event_router_url.as_deref() else {
        return;
    };
    ev.source = Some("pagi-executive-engine".to_string());
    let endpoint = format!("{}/publish", url.trim_end_matches('/'));
    let res = state.http.post(endpoint).json(&ev).send().await;
    if let Err(err) = res {
        tracing::warn!(error = %err, "failed to publish event");
    }
}

