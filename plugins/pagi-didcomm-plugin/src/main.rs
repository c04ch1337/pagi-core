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
use serde_json::{json, Value};
use std::{
    collections::HashMap,
    net::SocketAddr,
    path::{Path, PathBuf},
    sync::Arc,
};
use tokio::sync::RwLock;
use tracing::{error, info};
use uuid::Uuid;

#[derive(Clone)]
struct AppState {
    http: reqwest::Client,
    external_gateway_url: String,
    plugin_url: String,
    identity_keys_dir: PathBuf,
    inbox: Arc<RwLock<HashMap<String, Vec<SignedMessage>>>>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    pagi_http::tracing::init("pagi-didcomm-plugin");

    let external_gateway_url = std::env::var("EXTERNAL_GATEWAY_URL")
        .unwrap_or_else(|_| "http://127.0.0.1:8010".to_string());
    let plugin_url = std::env::var("PLUGIN_URL").unwrap_or_else(|_| "http://127.0.0.1:9030".to_string());
    let identity_keys_dir = std::env::var("IDENTITY_KEYS_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/data/identity/keys"));

    let state = AppState {
        http: reqwest::Client::new(),
        external_gateway_url,
        plugin_url,
        identity_keys_dir,
        inbox: Arc::new(RwLock::new(HashMap::new())),
    };

    // Best-effort: register tools with ExternalGateway on startup.
    let st = state.clone();
    tokio::spawn(async move {
        if let Err(err) = register_tools_with_gateway(&st).await {
            error!(error = %err, "failed to register DIDComm tools with ExternalGateway");
        }
    });

    let app = Router::new()
        .route("/healthz", get(|| async { "ok" }))
        // Tool endpoints (invoked via ExternalGateway)
        .route("/send", post(send_message))
        .route("/inbox", post(get_inbox))
        // Public receive endpoint for peers
        .route("/receive", post(receive_message))
        .with_state(state)
        .layer(tower_http::trace::TraceLayer::new_for_http());

    let addr: SocketAddr = pagi_http::config::bind_addr(([0, 0, 0, 0], 9030).into());
    info!(%addr, "pagi-didcomm-plugin listening");
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
    parameters: Value,
}

async fn register_tools_with_gateway(state: &AppState) -> Result<(), String> {
    let gateway = state.external_gateway_url.trim_end_matches('/');
    let register_url = format!("{gateway}/register_tool");

    let tools = vec![
        GatewayToolSchema {
            name: "didcomm_send_message".to_string(),
            description: "Send a signed peer message (DIDComm-like envelope; transport via HTTP)".to_string(),
            plugin_url: state.plugin_url.clone(),
            endpoint: "/send".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "from_twin_id": {"type": "string"},
                    "to_did": {"type": "string"},
                    "to_url": {"type": "string"},
                    "msg_type": {"type": "string"},
                    "body": {"type": "object"}
                },
                "required": ["from_twin_id", "to_did", "to_url", "msg_type", "body"]
            }),
        },
        GatewayToolSchema {
            name: "didcomm_get_inbox".to_string(),
            description: "Fetch and clear inbox for a DID".to_string(),
            plugin_url: state.plugin_url.clone(),
            endpoint: "/inbox".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {"did": {"type": "string"}},
                "required": ["did"]
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
    info!("registered DIDComm tools with ExternalGateway");
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SignedMessage {
    pub id: String,
    pub from_did: String,
    pub to_did: String,
    pub msg_type: String,
    pub body: Value,
    /// multibase(Base64Url) Ed25519 signature over the unsigned payload
    pub signature: String,
}

#[derive(Debug, Deserialize)]
struct SendRequest {
    pub from_twin_id: Uuid,
    pub to_did: String,
    pub to_url: String,
    pub msg_type: String,
    pub body: Value,
}

async fn send_message(State(state): State<AppState>, Json(req): Json<SendRequest>) -> impl IntoResponse {
    let (from_did, signature) = match sign_payload(&state.identity_keys_dir, req.from_twin_id, &req.to_did, &req.msg_type, &req.body)
    {
        Ok(v) => v,
        Err(e) => return (StatusCode::BAD_REQUEST, e).into_response(),
    };

    let msg = SignedMessage {
        id: Uuid::new_v4().to_string(),
        from_did,
        to_did: req.to_did,
        msg_type: req.msg_type,
        body: req.body,
        signature,
    };

    let url = format!("{}/receive", req.to_url.trim_end_matches('/'));
    match state.http.post(url).json(&msg).send().await {
        Ok(resp) if resp.status().is_success() => (StatusCode::OK, "sent").into_response(),
        Ok(resp) => (StatusCode::BAD_GATEWAY, format!("peer returned {}", resp.status())).into_response(),
        Err(e) => (StatusCode::BAD_GATEWAY, e.to_string()).into_response(),
    }
}

async fn receive_message(State(state): State<AppState>, Json(msg): Json<SignedMessage>) -> impl IntoResponse {
    if msg.to_did.trim().is_empty() {
        return (StatusCode::BAD_REQUEST, "to_did required").into_response();
    }

    match verify_payload(&msg) {
        Ok(true) => {
            let mut inbox = state.inbox.write().await;
            inbox.entry(msg.to_did.clone()).or_default().push(msg);
            StatusCode::ACCEPTED.into_response()
        }
        Ok(false) => (StatusCode::UNAUTHORIZED, "invalid signature").into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, e).into_response(),
    }
}

#[derive(Debug, Deserialize)]
struct InboxRequest {
    pub did: String,
}

async fn get_inbox(State(state): State<AppState>, Json(req): Json<InboxRequest>) -> impl IntoResponse {
    let mut inbox = state.inbox.write().await;
    let msgs = inbox.remove(&req.did).unwrap_or_default();
    Json(json!({"did": req.did, "messages": msgs})).into_response()
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

fn unsigned_payload(from_did: &str, to_did: &str, msg_type: &str, body: &Value) -> Result<Vec<u8>, String> {
    serde_json::to_vec(&json!({
        "from_did": from_did,
        "to_did": to_did,
        "msg_type": msg_type,
        "body": body,
    }))
    .map_err(|e| e.to_string())
}

fn sign_payload(
    identity_keys_dir: &Path,
    from_twin_id: Uuid,
    to_did: &str,
    msg_type: &str,
    body: &Value,
) -> Result<(String, String), String> {
    let signing_key = read_signing_key(identity_keys_dir, from_twin_id)?;
    let verifying_key = signing_key.verifying_key();
    let from_did = did_from_public_key(&verifying_key);
    let bytes = unsigned_payload(&from_did, to_did, msg_type, body)?;
    let sig: Signature = signing_key.sign(&bytes);
    let signature = multibase::encode(Base::Base64Url, sig.to_bytes());
    Ok((from_did, signature))
}

fn verify_payload(msg: &SignedMessage) -> Result<bool, String> {
    let verifying_key = verifying_key_from_did(&msg.from_did)?;
    let bytes = unsigned_payload(&msg.from_did, &msg.to_did, &msg.msg_type, &msg.body)?;
    let (_base, sig_bytes) = multibase::decode(&msg.signature).map_err(|e| format!("signature multibase decode failed: {e}"))?;
    let sig = Signature::from_slice(&sig_bytes).map_err(|e| e.to_string())?;
    Ok(verifying_key.verify(&bytes, &sig).is_ok())
}
