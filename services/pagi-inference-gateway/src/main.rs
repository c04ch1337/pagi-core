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
        event_router_url: std::env::var("EVENT_ROUTER_URL").ok(),
        http: reqwest::Client::new(),
    };

    let app = Router::new()
        .route("/healthz", get(healthz))
        .route("/infer", post(infer))
        .with_state(state)
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive());

    let addr: SocketAddr = pagi_http::config::bind_addr(([0, 0, 0, 0], 7005).into());
    tracing::info!(%addr, "listening");
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn healthz() -> (StatusCode, &'static str) {
    (StatusCode::OK, "ok")
}

async fn infer(State(state): State<AppState>, Json(req): Json<InferRequest>) -> Result<Json<InferResponse>, (StatusCode, String)> {
    publish_event(
        &state,
        EventEnvelope::new(
            EventType::InferenceRequested,
            json!({"twin_id": req.twin_id, "has_context": req.context.is_some()}),
        ),
    )
    .await;

    // MVP mock model adapter: returns a deterministic response.
    let output = if let Some(ctx) = &req.context {
        format!("[mock-model] Context:\n{}\n\nInput:\n{}", ctx, req.input)
    } else {
        format!("[mock-model] Input:\n{}", req.input)
    };

    publish_event(
        &state,
        EventEnvelope::new(
            EventType::InferenceCompleted,
            json!({"twin_id": req.twin_id, "output_len": output.len()}),
        ),
    )
    .await;

    Ok(Json(InferResponse {
        twin_id: req.twin_id,
        model: "mock".to_string(),
        output,
    }))
}

async fn publish_event(state: &AppState, mut ev: EventEnvelope) {
    let Some(url) = state.event_router_url.as_deref() else {
        return;
    };
    ev.source = Some("pagi-inference-gateway".to_string());
    let endpoint = format!("{}/publish", url.trim_end_matches('/'));
    let res = state.http.post(endpoint).json(&ev).send().await;
    if let Err(err) = res {
        tracing::warn!(error = %err, "failed to publish event");
    }
}

