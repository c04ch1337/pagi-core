use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, patch, post},
    Json, Router,
};
use pagi_common::{EventEnvelope, EventType, TwinId, TwinState};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{collections::HashMap, net::SocketAddr, sync::Arc};
use tokio::sync::RwLock;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use uuid::Uuid;

#[derive(Clone)]
struct AppState {
    twins: Arc<RwLock<HashMap<Uuid, TwinState>>>,
    event_router_url: Option<String>,
    http: reqwest::Client,
}

#[derive(Debug, Deserialize)]
struct CreateTwinRequest {
    #[serde(default)]
    pub initial_state: Option<TwinState>,
}

#[derive(Debug, Serialize)]
struct CreateTwinResponse {
    pub twin_id: TwinId,
    pub state: TwinState,
}

#[derive(Debug, Deserialize)]
struct UpdateStateRequest {
    pub state: TwinState,
}

#[tokio::main]
async fn main() {
    pagi_http::tracing::init("pagi-identity-service");

    let state = AppState {
        twins: Arc::new(RwLock::new(HashMap::new())),
        event_router_url: std::env::var("EVENT_ROUTER_URL").ok(),
        http: reqwest::Client::new(),
    };

    let app = Router::new()
        .route("/healthz", get(healthz))
        .route("/twins", post(create_twin))
        .route("/twins/:id", get(get_twin))
        .route("/twins/:id/state", patch(update_state))
        .with_state(state)
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive());

    let addr: SocketAddr = pagi_http::config::bind_addr(([0, 0, 0, 0], 7002).into());
    tracing::info!(%addr, "listening");
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn healthz() -> (StatusCode, &'static str) {
    (StatusCode::OK, "ok")
}

async fn create_twin(State(state): State<AppState>, Json(req): Json<CreateTwinRequest>) -> (StatusCode, Json<CreateTwinResponse>) {
    let id = Uuid::new_v4();
    let twin_state = req.initial_state.unwrap_or_default();
    state.twins.write().await.insert(id, twin_state.clone());

    publish_event(
        &state,
        EventEnvelope::new(
            EventType::TwinRegistered,
            json!({"twin_id": id, "state": twin_state}),
        ),
    )
    .await;

    (
        StatusCode::CREATED,
        Json(CreateTwinResponse {
            twin_id: TwinId(id),
            state: twin_state,
        }),
    )
}

async fn get_twin(State(state): State<AppState>, Path(id): Path<Uuid>) -> Result<Json<TwinState>, StatusCode> {
    let Some(st) = state.twins.read().await.get(&id).cloned() else {
        return Err(StatusCode::NOT_FOUND);
    };
    Ok(Json(st))
}

async fn update_state(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateStateRequest>,
) -> Result<(StatusCode, Json<TwinState>), StatusCode> {
    let mut guard = state.twins.write().await;
    let Some(entry) = guard.get_mut(&id) else {
        return Err(StatusCode::NOT_FOUND);
    };
    *entry = req.state.clone();

    publish_event(
        &state,
        EventEnvelope::new(EventType::TwinStateUpdated, json!({"twin_id": id, "state": entry})),
    )
    .await;

    Ok((StatusCode::OK, Json(entry.clone())))
}

async fn publish_event(state: &AppState, mut ev: EventEnvelope) {
    let Some(url) = state.event_router_url.as_deref() else {
        return;
    };
    ev.source = Some("pagi-identity-service".to_string());
    let endpoint = format!("{}/publish", url.trim_end_matches('/'));
    let res = state.http.post(endpoint).json(&ev).send().await;
    if let Err(err) = res {
        tracing::warn!(error = %err, "failed to publish event");
    }
}

