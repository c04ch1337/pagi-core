use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use base64::Engine;
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use multibase::Base;
use pagi_common::{PagiError, TwinId};
use pagi_http::errors::PagiAxumError;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    collections::HashMap,
    net::SocketAddr,
    path::{Path, PathBuf},
    sync::Arc,
};
use tokio::sync::Mutex;
use tokio::sync::RwLock;
use tracing::{error, info};
use uuid::Uuid;

#[derive(Clone)]
struct AppState {
    http: reqwest::Client,
    external_gateway_url: String,
    plugin_url: String,
    identity_keys_dir: PathBuf,
    mailbox: Mailbox,
}

#[derive(Clone)]
struct Mailbox {
    /// Fast-path in-memory store. Used for both normal nodes and relay nodes.
    mem: Arc<RwLock<HashMap<String, Vec<SignedMessage>>>>,
    /// Optional persistence directory. If set, messages are also appended to disk and will survive restarts.
    dir: Option<PathBuf>,
    file_lock: Arc<Mutex<()>>,
    max_per_did: usize,
}

impl Mailbox {
    fn new(dir: Option<PathBuf>, max_per_did: usize) -> Self {
        Self {
            mem: Arc::new(RwLock::new(HashMap::new())),
            dir,
            file_lock: Arc::new(Mutex::new(())),
            max_per_did,
        }
    }

    fn did_to_filename(did: &str) -> String {
        // DID strings contain ":" and other characters that are annoying in filenames.
        // Base64URL(NO_PAD) yields a portable filename component.
        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(did.as_bytes())
    }

    async fn put(&self, did: &str, mut msg: SignedMessage) {
        msg.relay_received_at = Some(time::OffsetDateTime::now_utc().unix_timestamp());
        {
            let mut mem = self.mem.write().await;
            let q = mem.entry(did.to_string()).or_default();
            q.push(msg.clone());
            if q.len() > self.max_per_did {
                // Drop oldest.
                let overflow = q.len() - self.max_per_did;
                q.drain(0..overflow);
            }
        }

        let Some(dir) = self.dir.as_ref() else { return; };

        // Serialize file operations to avoid interleaving writes.
        let _guard = self.file_lock.lock().await;
        if let Err(err) = tokio::fs::create_dir_all(dir).await {
            tracing::warn!(error = %err, "failed to create mailbox dir");
            return;
        }

        let path = dir.join(format!("{}.jsonl", Self::did_to_filename(did)));
        let line = match serde_json::to_string(&msg) {
            Ok(v) => v,
            Err(err) => {
                tracing::warn!(error = %err, "failed to serialize mailbox message");
                return;
            }
        };

        match tokio::fs::OpenOptions::new().create(true).append(true).open(&path).await {
            Ok(mut f) => {
                use tokio::io::AsyncWriteExt;
                if let Err(err) = f.write_all(line.as_bytes()).await {
                    tracing::warn!(error = %err, path = %path.display(), "failed to append mailbox message");
                    return;
                }
                if let Err(err) = f.write_all(b"\n").await {
                    tracing::warn!(error = %err, path = %path.display(), "failed to append mailbox message");
                }
            }
            Err(err) => {
                tracing::warn!(error = %err, path = %path.display(), "failed to open mailbox file");
            }
        }
    }

    async fn take_all(&self, did: &str) -> Vec<SignedMessage> {
        let mut out = {
            let mut mem = self.mem.write().await;
            mem.remove(did).unwrap_or_default()
        };

        let Some(dir) = self.dir.as_ref() else {
            return out;
        };

        let _guard = self.file_lock.lock().await;
        let path = dir.join(format!("{}.jsonl", Self::did_to_filename(did)));
        let Ok(text) = tokio::fs::read_to_string(&path).await else {
            return out;
        };

        // Best-effort: delete file (mailbox semantics).
        let _ = tokio::fs::remove_file(&path).await;

        for line in text.lines() {
            if line.trim().is_empty() {
                continue;
            }
            match serde_json::from_str::<SignedMessage>(line) {
                Ok(m) => out.push(m),
                Err(err) => tracing::warn!(error = %err, "failed to parse mailbox line"),
            }
        }

        out
    }
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

    // If set, this node will persist mailbox messages to disk (suitable for always-on relays).
    // Example: DIDCOMM_MAILBOX_DIR=/data/didcomm-mailbox
    let mailbox_dir = std::env::var("DIDCOMM_MAILBOX_DIR").ok().filter(|s| !s.trim().is_empty()).map(PathBuf::from);
    let max_per_did = std::env::var("DIDCOMM_MAILBOX_MAX_PER_DID")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(10_000);

    let state = AppState {
        http: reqwest::Client::new(),
        external_gateway_url,
        plugin_url,
        identity_keys_dir,
        mailbox: Mailbox::new(mailbox_dir, max_per_did),
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
        .route("/send_with_relay", post(send_message_with_relay))
        .route("/poll_relay", post(poll_relay))
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
            name: "didcomm_send_message_with_relay".to_string(),
            description: "Send a signed peer message. If direct delivery fails, store-and-forward via a relay node (HTTP mailbox).".to_string(),
            plugin_url: state.plugin_url.clone(),
            endpoint: "/send_with_relay".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "from_twin_id": {"type": "string"},
                    "to_did": {"type": "string"},
                    "to_url": {"type": "string"},
                    "relay_url": {"type": "string"},
                    "msg_type": {"type": "string"},
                    "body": {"type": "object"}
                },
                "required": ["from_twin_id", "to_did", "to_url", "relay_url", "msg_type", "body"]
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
        GatewayToolSchema {
            name: "didcomm_poll_relay_inbox".to_string(),
            description: "Poll a relay node mailbox for a DID (store-and-forward). Returns and clears messages on the relay.".to_string(),
            plugin_url: state.plugin_url.clone(),
            endpoint: "/poll_relay".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "did": {"type": "string"},
                    "relay_url": {"type": "string"}
                },
                "required": ["did", "relay_url"]
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

    /// Unix timestamp (seconds) when a relay node accepted the message.
    /// Optional so older senders remain compatible.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub relay_received_at: Option<i64>,
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
        Err(e) => {
            return PagiAxumError::with_status(PagiError::config(e), StatusCode::BAD_REQUEST).into_response();
        }
    };

    let msg = SignedMessage {
        id: Uuid::new_v4().to_string(),
        from_did,
        to_did: req.to_did,
        msg_type: req.msg_type,
        body: req.body,
        signature,
        relay_received_at: None,
    };

    let url = format!("{}/receive", req.to_url.trim_end_matches('/'));
    match state.http.post(url).json(&msg).send().await {
        Ok(resp) if resp.status().is_success() => (StatusCode::OK, "sent").into_response(),
        Ok(resp) => PagiAxumError::with_status(
            PagiError::plugin_exec(format!("peer returned {}", resp.status())),
            StatusCode::BAD_GATEWAY,
        )
        .into_response(),
        Err(e) => PagiAxumError::with_status(PagiError::from(e), StatusCode::BAD_GATEWAY).into_response(),
    }
}

#[derive(Debug, Deserialize)]
struct SendWithRelayRequest {
    pub from_twin_id: Uuid,
    pub to_did: String,
    /// Recipient base URL for direct send (expects POST {base}/receive)
    pub to_url: String,
    /// Relay base URL used as a mailbox when the recipient is offline/unreachable.
    /// The relay must be running this same DIDComm plugin and expose POST {base}/receive.
    pub relay_url: String,
    pub msg_type: String,
    pub body: Value,
}

async fn send_message_with_relay(
    State(state): State<AppState>,
    Json(req): Json<SendWithRelayRequest>,
) -> impl IntoResponse {
    let (from_did, signature) = match sign_payload(
        &state.identity_keys_dir,
        req.from_twin_id,
        &req.to_did,
        &req.msg_type,
        &req.body,
    ) {
        Ok(v) => v,
        Err(e) => {
            return PagiAxumError::with_status(PagiError::config(e), StatusCode::BAD_REQUEST).into_response();
        }
    };

    let msg = SignedMessage {
        id: Uuid::new_v4().to_string(),
        from_did,
        to_did: req.to_did,
        msg_type: req.msg_type,
        body: req.body,
        signature,
        relay_received_at: None,
    };

    // Attempt direct delivery first.
    let direct_url = format!("{}/receive", req.to_url.trim_end_matches('/'));
    match state.http.post(direct_url).json(&msg).send().await {
        Ok(resp) if resp.status().is_success() => return (StatusCode::OK, "sent").into_response(),
        Ok(resp) => {
            // Fall through to relay (peer returned non-2xx).
            tracing::warn!(status = %resp.status(), "direct didcomm send failed; attempting relay");
        }
        Err(err) => {
            tracing::warn!(error = %err, "direct didcomm send error; attempting relay");
        }
    }

    // Store-and-forward via relay: deliver to relay's /receive which will persist in its inbox keyed by to_did.
    let relay_receive_url = format!("{}/receive", req.relay_url.trim_end_matches('/'));
    match state.http.post(relay_receive_url).json(&msg).send().await {
        Ok(resp) if resp.status().is_success() => (StatusCode::ACCEPTED, "relayed").into_response(),
        Ok(resp) => PagiAxumError::with_status(
            PagiError::plugin_exec(format!("relay returned {}", resp.status())),
            StatusCode::BAD_GATEWAY,
        )
        .into_response(),
        Err(e) => PagiAxumError::with_status(PagiError::from(e), StatusCode::BAD_GATEWAY).into_response(),
    }
}

async fn receive_message(State(state): State<AppState>, Json(msg): Json<SignedMessage>) -> impl IntoResponse {
    if msg.to_did.trim().is_empty() {
        return PagiAxumError::with_status(PagiError::config("to_did required"), StatusCode::BAD_REQUEST).into_response();
    }

    match verify_payload(&msg) {
        Ok(true) => {
            let did = msg.to_did.clone();
            state.mailbox.put(&did, msg).await;
            StatusCode::ACCEPTED.into_response()
        }
        Ok(false) => (StatusCode::UNAUTHORIZED, "invalid signature").into_response(),
        Err(e) => PagiAxumError::with_status(PagiError::config(e), StatusCode::BAD_REQUEST).into_response(),
    }
}

#[derive(Debug, Deserialize)]
struct InboxRequest {
    pub did: String,
}

async fn get_inbox(State(state): State<AppState>, Json(req): Json<InboxRequest>) -> impl IntoResponse {
    let msgs = state.mailbox.take_all(&req.did).await;
    Json(json!({"did": req.did, "messages": msgs})).into_response()
}

#[derive(Debug, Deserialize)]
struct PollRelayRequest {
    pub did: String,
    pub relay_url: String,
}

async fn poll_relay(State(state): State<AppState>, Json(req): Json<PollRelayRequest>) -> impl IntoResponse {
    let relay_inbox_url = format!("{}/inbox", req.relay_url.trim_end_matches('/'));
    let payload = json!({"did": req.did});

    match state.http.post(relay_inbox_url).json(&payload).send().await {
        Ok(resp) if resp.status().is_success() => match resp.json::<serde_json::Value>().await {
            Ok(v) => Json(v).into_response(),
            Err(e) => PagiAxumError::with_status(PagiError::from(e), StatusCode::BAD_GATEWAY).into_response(),
        },
        Ok(resp) => PagiAxumError::with_status(
            PagiError::plugin_exec(format!("relay returned {}", resp.status())),
            StatusCode::BAD_GATEWAY,
        )
        .into_response(),
        Err(e) => PagiAxumError::with_status(PagiError::from(e), StatusCode::BAD_GATEWAY).into_response(),
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
