use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, patch, post},
    Json, Router,
};
use ed25519_dalek::SigningKey;
use multibase::Base;
use pagi_common::{publish_event, EventEnvelope, EventType, TwinId, TwinState};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{collections::HashMap, net::SocketAddr, sync::Arc};
use tokio::sync::RwLock;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use uuid::Uuid;

#[derive(Clone)]
struct AppState {
    twins: Arc<RwLock<HashMap<Uuid, TwinState>>>,
    identities: Arc<RwLock<HashMap<Uuid, TwinIdentity>>>,
}

#[derive(Debug, Clone, Serialize)]
struct TwinIdentity {
    did: String,
    did_document: serde_json::Value,
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
    pub did: String,
    pub did_document: serde_json::Value,
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
        identities: Arc::new(RwLock::new(HashMap::new())),
    };

    let app = Router::new()
        .route("/healthz", get(healthz))
        .route("/twins", post(create_twin))
        .route("/twins/:id", get(get_twin))
        .route("/twins/:id/did", get(get_did))
        .route("/twins/:id/state", patch(update_state))
        .with_state(state)
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive());

    let addr: SocketAddr = pagi_http::config::bind_addr(([0, 0, 0, 0], 8002).into());
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

    // Generate DID (did:key using Ed25519) + persist private key to disk.
    let (did, did_document) = match create_and_persist_did(id) {
        Ok(v) => v,
        Err(err) => {
            tracing::error!(twin_id = %id, error = %err, "failed to create DID");
            (format!("did:key:unavailable:{}", id), json!({"error": err}))
        }
    };
    state
        .identities
        .write()
        .await
        .insert(id, TwinIdentity { did: did.clone(), did_document: did_document.clone() });

    let mut ev = EventEnvelope::new(
        EventType::TwinRegistered,
        json!({"twin_id": id, "state": twin_state}),
    );
    ev.twin_id = Some(id);
    ev.source = Some("pagi-identity-service".to_string());
    let _ = publish_event(ev).await;

    (
        StatusCode::CREATED,
        Json(CreateTwinResponse {
            twin_id: TwinId(id),
            state: twin_state,
            did,
            did_document,
        }),
    )
}

async fn get_did(State(state): State<AppState>, Path(id): Path<Uuid>) -> Result<Json<serde_json::Value>, StatusCode> {
    let Some(ident) = state.identities.read().await.get(&id).cloned() else {
        return Err(StatusCode::NOT_FOUND);
    };
    Ok(Json(ident.did_document))
}

fn create_and_persist_did(twin_uuid: Uuid) -> Result<(String, serde_json::Value), String> {
    use rand_core::OsRng;

    let mut rng = OsRng;
    let signing_key = SigningKey::generate(&mut rng);
    let verifying_key = signing_key.verifying_key();

    let public_bytes = verifying_key.to_bytes();
    let secret_bytes = signing_key.to_bytes();

    // did:key method-specific-id is multibase(base58btc, multicodec(ed25519-pub) || pubkey)
    // Multicodec prefix for Ed25519 public key is 0xed 0x01.
    let mut codec_and_key = Vec::with_capacity(2 + public_bytes.len());
    codec_and_key.push(0xed);
    codec_and_key.push(0x01);
    codec_and_key.extend_from_slice(&public_bytes);

    let method_id = multibase::encode(Base::Base58Btc, codec_and_key);
    let did = format!("did:key:{method_id}");

    let vm_id = format!("{}#{}", did, method_id);
    let did_document = json!({
        "@context": "https://www.w3.org/ns/did/v1",
        "id": did,
        "verificationMethod": [{
            "id": vm_id,
            "type": "Ed25519VerificationKey2020",
            "controller": did,
            "publicKeyMultibase": method_id,
        }],
        "authentication": [vm_id],
        "assertionMethod": [vm_id],
    });

    // Persist the private key for later signing.
    // NOTE: This is intentionally minimal. Phase 5 should encrypt-at-rest.
    let data_dir = std::env::var("IDENTITY_DATA_DIR").unwrap_or_else(|_| "/data/identity".to_string());
    let keys_dir = std::path::PathBuf::from(data_dir).join("keys");
    std::fs::create_dir_all(&keys_dir).map_err(|e| e.to_string())?;
    let key_path = keys_dir.join(format!("{twin_uuid}.ed25519"));
    std::fs::write(&key_path, secret_bytes).map_err(|e| e.to_string())?;

    Ok((did_document["id"].as_str().unwrap_or_default().to_string(), did_document))
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

    let mut ev = EventEnvelope::new(EventType::TwinStateUpdated, json!({"twin_id": id, "state": entry}));
    ev.twin_id = Some(id);
    ev.source = Some("pagi-identity-service".to_string());
    let _ = publish_event(ev).await;

    Ok((StatusCode::OK, Json(entry.clone())))
}
