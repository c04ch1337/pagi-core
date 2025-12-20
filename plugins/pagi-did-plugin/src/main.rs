use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use multibase::Base;
use pagi_common::TwinId;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{
    net::SocketAddr,
    path::{Path, PathBuf},
};
use tracing::{error, info};
use uuid::Uuid;

#[derive(Clone)]
struct AppState {
    http: reqwest::Client,
    external_gateway_url: String,
    plugin_url: String,
    identity_keys_dir: PathBuf,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    pagi_http::tracing::init("pagi-did-plugin");

    let external_gateway_url = std::env::var("EXTERNAL_GATEWAY_URL")
        .unwrap_or_else(|_| "http://127.0.0.1:8010".to_string());
    let plugin_url = std::env::var("PLUGIN_URL").unwrap_or_else(|_| "http://127.0.0.1:9020".to_string());
    let identity_keys_dir = std::env::var("IDENTITY_KEYS_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/data/identity/keys"));

    let state = AppState {
        http: reqwest::Client::new(),
        external_gateway_url,
        plugin_url,
        identity_keys_dir,
    };

    // Best-effort: register tools with ExternalGateway on startup.
    let st = state.clone();
    tokio::spawn(async move {
        if let Err(err) = register_tools_with_gateway(&st).await {
            error!(error = %err, "failed to register DID tools with ExternalGateway");
        }
    });

    let app = Router::new()
        .route("/healthz", get(|| async { "ok" }))
        .route("/sign_artifact", post(sign_artifact))
        .route("/verify_artifact", post(verify_artifact))
        .with_state(state)
        .layer(tower_http::trace::TraceLayer::new_for_http());

    let addr: SocketAddr = pagi_http::config::bind_addr(([0, 0, 0, 0], 9020).into());
    info!(%addr, "pagi-did-plugin listening");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GatewayRegisterPayload {
    twin_id: Option<TwinId>,
    tool: GatewayToolSchema,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GatewayToolSchema {
    name: String,
    description: String,
    plugin_url: String,
    endpoint: String,
    parameters: serde_json::Value,
}

async fn register_tools_with_gateway(state: &AppState) -> Result<(), String> {
    let gateway = state.external_gateway_url.trim_end_matches('/');
    let register_url = format!("{gateway}/register_tool");

    let tools = vec![
        GatewayToolSchema {
            name: "did_sign_artifact".to_string(),
            description: "Sign an artifact payload using the twin's Ed25519 DID key".to_string(),
            plugin_url: state.plugin_url.clone(),
            endpoint: "/sign_artifact".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "twin_id": {"type": "string"},
                    "artifact": {"type": "object"}
                },
                "required": ["twin_id", "artifact"]
            }),
        },
        GatewayToolSchema {
            name: "did_verify_artifact".to_string(),
            description: "Verify a signed artifact using the signer's did:key".to_string(),
            plugin_url: state.plugin_url.clone(),
            endpoint: "/verify_artifact".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "did": {"type": "string"},
                    "signature": {"type": "string"},
                    "artifact": {"type": "object"}
                },
                "required": ["did", "signature", "artifact"]
            }),
        },
    ];

    for tool in tools {
        let payload = GatewayRegisterPayload { twin_id: None, tool };
        state
            .http
            .post(&register_url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| e.to_string())?
            .error_for_status()
            .map_err(|e| e.to_string())?;
    }

    info!("registered DID tools with ExternalGateway");
    Ok(())
}

#[derive(Debug, Deserialize)]
struct SignRequest {
    pub twin_id: Uuid,
    pub artifact: serde_json::Value,
}

#[derive(Debug, Serialize)]
struct SignResponse {
    pub did: String,
    pub signature: String,
    pub artifact: serde_json::Value,
}

async fn sign_artifact(State(state): State<AppState>, Json(req): Json<SignRequest>) -> impl IntoResponse {
    match sign_with_twin_key(&state.identity_keys_dir, req.twin_id, &req.artifact) {
        Ok((did, signature)) => (StatusCode::OK, Json(SignResponse { did, signature, artifact: req.artifact })).into_response(),
        Err(err) => (StatusCode::BAD_REQUEST, err).into_response(),
    }
}

#[derive(Debug, Deserialize)]
struct VerifyRequest {
    pub did: String,
    pub signature: String,
    pub artifact: serde_json::Value,
}

#[derive(Debug, Serialize)]
struct VerifyResponse {
    pub valid: bool,
}

async fn verify_artifact(Json(req): Json<VerifyRequest>) -> impl IntoResponse {
    match verify_with_did_key(&req.did, &req.signature, &req.artifact) {
        Ok(valid) => (StatusCode::OK, Json(VerifyResponse { valid })).into_response(),
        Err(err) => (StatusCode::BAD_REQUEST, err).into_response(),
    }
}

fn read_signing_key(identity_keys_dir: &Path, twin_id: Uuid) -> Result<SigningKey, String> {
    let key_path = identity_keys_dir.join(format!("{twin_id}.ed25519"));
    let raw = std::fs::read(&key_path).map_err(|e| format!("failed to read key {key_path:?}: {e}"))?;
    if raw.len() != 32 {
        return Err(format!("invalid key length: expected 32 bytes, got {}", raw.len()));
    }
    let mut sk = [0u8; 32];
    sk.copy_from_slice(&raw);
    Ok(SigningKey::from_bytes(&sk))
}

fn did_from_public_key(public_key: &VerifyingKey) -> String {
    let pub_bytes = public_key.to_bytes();
    let mut codec_and_key = Vec::with_capacity(2 + pub_bytes.len());
    codec_and_key.push(0xed);
    codec_and_key.push(0x01);
    codec_and_key.extend_from_slice(&pub_bytes);
    let method_id = multibase::encode(Base::Base58Btc, codec_and_key);
    format!("did:key:{method_id}")
}

fn verifying_key_from_did(did: &str) -> Result<VerifyingKey, String> {
    let method_id = did
        .strip_prefix("did:key:")
        .ok_or_else(|| "did must start with did:key:".to_string())?;

    let (_base, bytes) = multibase::decode(method_id).map_err(|e| format!("multibase decode failed: {e}"))?;
    if bytes.len() != 2 + 32 {
        return Err(format!("unexpected did:key decoded length: {}", bytes.len()));
    }
    if bytes[0] != 0xed || bytes[1] != 0x01 {
        return Err("unsupported key type (expected ed25519-pub multicodec 0xed01)".to_string());
    }
    let mut pk = [0u8; 32];
    pk.copy_from_slice(&bytes[2..]);
    VerifyingKey::from_bytes(&pk).map_err(|e| e.to_string())
}

fn sign_with_twin_key(identity_keys_dir: &Path, twin_id: Uuid, artifact: &serde_json::Value) -> Result<(String, String), String> {
    let signing_key = read_signing_key(identity_keys_dir, twin_id)?;
    let verifying_key = signing_key.verifying_key();
    let did = did_from_public_key(&verifying_key);

    let msg = serde_json::to_vec(artifact).map_err(|e| e.to_string())?;
    let sig: Signature = signing_key.sign(&msg);
    let signature = multibase::encode(Base::Base64Url, sig.to_bytes());
    Ok((did, signature))
}

fn verify_with_did_key(did: &str, signature: &str, artifact: &serde_json::Value) -> Result<bool, String> {
    let verifying_key = verifying_key_from_did(did)?;
    let (_base, sig_bytes) = multibase::decode(signature).map_err(|e| format!("signature multibase decode failed: {e}"))?;
    let sig = Signature::from_slice(&sig_bytes).map_err(|e| e.to_string())?;
    let msg = serde_json::to_vec(artifact).map_err(|e| e.to_string())?;
    Ok(verifying_key.verify(&msg, &sig).is_ok())
}
