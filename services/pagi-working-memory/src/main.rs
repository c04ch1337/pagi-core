use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use pagi_common::{EventEnvelope, EventType};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{collections::HashMap, net::SocketAddr, sync::Arc};
use tokio::sync::RwLock;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use uuid::Uuid;

#[derive(Clone)]
struct AppState {
    mem: Arc<RwLock<HashMap<Uuid, Vec<MemoryItem>>>>,
    event_router_url: Option<String>,
    http: reqwest::Client,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryItem {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Deserialize)]
struct AppendRequest {
    pub item: MemoryItem,
}

#[tokio::main]
async fn main() {
    pagi_http::tracing::init("pagi-working-memory");

    let state = AppState {
        mem: Arc::new(RwLock::new(HashMap::new())),
        event_router_url: std::env::var("EVENT_ROUTER_URL").ok(),
        http: reqwest::Client::new(),
    };

    let app = Router::new()
        .route("/healthz", get(healthz))
        .route("/memory/:twin_id", get(get_memory))
        .route("/memory/:twin_id/append", post(append_memory))
        .with_state(state)
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive());

    let addr: SocketAddr = pagi_http::config::bind_addr(([0, 0, 0, 0], 7003).into());
    tracing::info!(%addr, "listening");
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn healthz() -> (StatusCode, &'static str) {
    (StatusCode::OK, "ok")
}

async fn get_memory(State(state): State<AppState>, Path(twin_id): Path<Uuid>) -> Json<Vec<MemoryItem>> {
    let guard = state.mem.read().await;
    let items = guard.get(&twin_id).cloned().unwrap_or_default();
    Json(items)
}

async fn append_memory(
    State(state): State<AppState>,
    Path(twin_id): Path<Uuid>,
    Json(req): Json<AppendRequest>,
) -> (StatusCode, Json<Vec<MemoryItem>>) {
    let mut guard = state.mem.write().await;
    let entry = guard.entry(twin_id).or_default();
    entry.push(req.item.clone());

    publish_event(
        &state,
        EventEnvelope::new(
            EventType::WorkingMemoryAppended,
            json!({"twin_id": twin_id, "item": req.item}),
        ),
    )
    .await;

    (StatusCode::OK, Json(entry.clone()))
}

async fn publish_event(state: &AppState, mut ev: EventEnvelope) {
    let Some(url) = state.event_router_url.as_deref() else {
        return;
    };
    ev.source = Some("pagi-working-memory".to_string());
    let endpoint = format!("{}/publish", url.trim_end_matches('/'));
    let res = state.http.post(endpoint).json(&ev).send().await;
    if let Err(err) = res {
        tracing::warn!(error = %err, "failed to publish event");
    }
}

