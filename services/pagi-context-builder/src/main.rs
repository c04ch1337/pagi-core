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
    working_memory_url: String,
    event_router_url: Option<String>,
    http: reqwest::Client,
}

#[derive(Debug, Deserialize)]
struct BuildRequest {
    pub twin_id: Uuid,
    pub query: String,
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
            .unwrap_or_else(|_| "http://127.0.0.1:7003".to_string()),
        event_router_url: std::env::var("EVENT_ROUTER_URL").ok(),
        http: reqwest::Client::new(),
    };

    let app = Router::new()
        .route("/healthz", get(healthz))
        .route("/build", post(build_context))
        .with_state(state)
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive());

    let addr: SocketAddr = pagi_http::config::bind_addr(([0, 0, 0, 0], 7004).into());
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

    let mut context = String::new();
    context.push_str("# Working Memory\n");
    for item in &mem {
        let role = item.get("role").and_then(|v| v.as_str()).unwrap_or("unknown");
        let content = item
            .get("content")
            .and_then(|v| v.as_str())
            .unwrap_or_default();
        context.push_str(&format!("- {}: {}\n", role, content));
    }
    context.push_str("\n# Query\n");
    context.push_str(&req.query);

    let resp = BuildResponse {
        twin_id: req.twin_id,
        context,
        sources: vec!["working_memory".to_string()],
    };

    publish_event(
        &state,
        EventEnvelope::new(EventType::ContextBuilt, json!({"twin_id": req.twin_id})),
    )
    .await;

    Ok(Json(resp))
}

async fn publish_event(state: &AppState, mut ev: EventEnvelope) {
    let Some(url) = state.event_router_url.as_deref() else {
        return;
    };
    ev.source = Some("pagi-context-builder".to_string());
    let endpoint = format!("{}/publish", url.trim_end_matches('/'));
    let res = state.http.post(endpoint).json(&ev).send().await;
    if let Err(err) = res {
        tracing::warn!(error = %err, "failed to publish event");
    }
}

