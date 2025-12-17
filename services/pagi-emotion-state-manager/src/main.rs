use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::get,
    Json, Router,
};
use pagi_common::{publish_event, EventEnvelope, EventType};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{collections::HashMap, net::SocketAddr, sync::Arc};
use tokio::sync::RwLock;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct EmotionState {
    pub mood: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stress: Option<f32>,
}

impl Default for EmotionState {
    fn default() -> Self {
        Self {
            mood: "neutral".to_string(),
            stress: None,
        }
    }
}

#[derive(Clone)]
struct AppState {
    store: Arc<RwLock<HashMap<Uuid, EmotionState>>>,
}

#[tokio::main]
async fn main() {
    pagi_http::tracing::init("pagi-emotion-state-manager");

    let state = AppState {
        store: Arc::new(RwLock::new(HashMap::new())),
    };

    let app = Router::new()
        .route("/healthz", get(healthz))
        .route("/emotion/:twin_id", get(get_state).put(set_state))
        .with_state(state)
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive());

    let addr: SocketAddr = pagi_http::config::bind_addr(([0, 0, 0, 0], 8007).into());
    tracing::info!(%addr, "listening");
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn healthz() -> (StatusCode, &'static str) {
    (StatusCode::OK, "ok")
}

async fn get_state(State(state): State<AppState>, Path(twin_id): Path<Uuid>) -> Json<EmotionState> {
    let guard = state.store.read().await;
    Json(guard.get(&twin_id).cloned().unwrap_or_default())
}

async fn set_state(
    State(state): State<AppState>,
    Path(twin_id): Path<Uuid>,
    Json(new_state): Json<EmotionState>,
) -> Json<EmotionState> {
    state.store.write().await.insert(twin_id, new_state.clone());

    let mut ev = EventEnvelope::new(
        EventType::EmotionStateUpdated,
        json!({"twin_id": twin_id, "mood": new_state.mood, "stress": new_state.stress}),
    );
    ev.twin_id = Some(twin_id);
    ev.source = Some("pagi-emotion-state-manager".to_string());
    let _ = publish_event(ev).await;

    Json(new_state)
}
