use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use pagi_common::{publish_event, EventEnvelope, EventType};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{collections::HashMap, net::SocketAddr, sync::Arc};
use tokio::sync::RwLock;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use uuid::Uuid;

#[derive(Clone)]
struct AppState {
    mem: Arc<RwLock<HashMap<Uuid, Vec<MemoryItem>>>>,
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
    };

    let app = Router::new()
        .route("/healthz", get(healthz))
        .route("/memory/:twin_id", get(get_memory))
        .route("/memory/:twin_id/append", post(append_memory))
        .with_state(state)
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive());

    let addr: SocketAddr = pagi_http::config::bind_addr(([0, 0, 0, 0], 8003).into());
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

    let mut ev = EventEnvelope::new(
        EventType::WorkingMemoryAppended,
        json!({"twin_id": twin_id, "item": req.item}),
    );
    ev.twin_id = Some(twin_id);
    ev.source = Some("pagi-working-memory".to_string());
    let _ = publish_event(ev).await;

    (StatusCode::OK, Json(entry.clone()))
}
