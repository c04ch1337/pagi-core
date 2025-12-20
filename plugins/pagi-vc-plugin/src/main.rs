use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use multibase::Base;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
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
    pagi_http::tracing::init("pagi-vc-plugin");

    let external_gateway_url = std::env::var("EXTERNAL_GATEWAY_URL")
        .unwrap_or_else(|_| "http://127.0.0.1:8010".to_string());
    let plugin_url = std::env::var("PLUGIN_URL").unwrap_or_else(|_| "http://127.0.0.1:9040".to_string());
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
            error!(error = %err, "failed to register VC tools with ExternalGateway");
        }
    });

    let app = Router::new()
        .route("/healthz", get(|| async { "ok" }))
        .route("/issue_reputation_vc", post(issue_reputation_vc))
        .route("/verify_vc", post(verify_vc))
        .with_state(state)
        .layer(tower_http::trace::TraceLayer::new_for_http());

    let addr: SocketAddr = pagi_http::config::bind_addr(([0, 0, 0, 0], 9040).into());
    info!(%addr, "pagi-vc-plugin listening");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GatewayRegisterPayload {
    twin_id: Option<pagi_common::TwinId>,
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
            name: "vc_issue_reputation".to_string(),
            description: "Issue a simple W3C-VC-like reputation credential signed by issuer did:key".to_string(),
            plugin_url: state.plugin_url.clone(),
            endpoint: "/issue_reputation_vc".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "issuer_twin_id": {"type": "string"},
                    "subject_did": {"type": "string"},
                    "reputation_score": {"type": "number"},
                    "contributions": {"type": "number"}
                },
                "required": ["issuer_twin_id", "subject_did", "reputation_score"]
            }),
        },
        GatewayToolSchema {
            name: "vc_verify".to_string(),
            description: "Verify a VC-like document with an Ed25519 proof from issuer did:key".to_string(),
            plugin_url: state.plugin_url.clone(),
            endpoint: "/verify_vc".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {"credential": {"type": "object"}},
                "required": ["credential"]
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
    info!("registered VC tools with ExternalGateway");
    Ok(())
}

#[derive(Debug, Deserialize)]
struct IssueReputationRequest {
    pub issuer_twin_id: Uuid,
    pub subject_did: String,
    pub reputation_score: f64,
    #[serde(default)]
    pub contributions: Option<f64>,
}

async fn issue_reputation_vc(State(state): State<AppState>, Json(req): Json<IssueReputationRequest>) -> impl IntoResponse {
    match issue_reputation_credential(&state.identity_keys_dir, &req) {
        Ok(vc) => (StatusCode::OK, Json(vc)).into_response(),
        Err(err) => (StatusCode::BAD_REQUEST, err).into_response(),
    }
}

#[derive(Debug, Deserialize)]
struct VerifyVcRequest {
    pub credential: Value,
}

async fn verify_vc(Json(req): Json<VerifyVcRequest>) -> impl IntoResponse {
    match verify_credential(&req.credential) {
        Ok(valid) => (StatusCode::OK, Json(json!({"valid": valid}))).into_response(),
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

fn issue_reputation_credential(identity_keys_dir: &Path, req: &IssueReputationRequest) -> Result<Value, String> {
    let signing_key = read_signing_key(identity_keys_dir, req.issuer_twin_id)?;
    let issuer_vk = signing_key.verifying_key();
    let issuer_did = did_from_public_key(&issuer_vk);
    let method_id = issuer_did
        .strip_prefix("did:key:")
        .ok_or_else(|| "issuer did is not did:key".to_string())?;
    let verification_method = format!("{}#{}", issuer_did, method_id);

    let now = time::OffsetDateTime::now_utc().format(&time::format_description::well_known::Rfc3339).unwrap_or_else(|_| "".to_string());

    let mut credential = json!({
        "@context": ["https://www.w3.org/ns/credentials/v2"],
        "type": ["VerifiableCredential", "PAGIReputationCredential"],
        "issuer": issuer_did,
        "validFrom": now,
        "credentialSubject": {
            "id": req.subject_did,
            "reputationScore": req.reputation_score,
        }
    });
    if let Some(c) = req.contributions {
        credential["credentialSubject"]["contributions"] = json!(c);
    }

    // Sign the credential without proof.
    let msg = serde_json::to_vec(&credential).map_err(|e| e.to_string())?;
    let sig: Signature = signing_key.sign(&msg);
    let proof_value = multibase::encode(Base::Base64Url, sig.to_bytes());

    credential["proof"] = json!({
        "type": "Ed25519Signature2020",
        "created": now,
        "proofPurpose": "assertionMethod",
        "verificationMethod": verification_method,
        "proofValue": proof_value,
    });

    Ok(credential)
}

fn verify_credential(credential: &Value) -> Result<bool, String> {
    let issuer = credential
        .get("issuer")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "credential.issuer missing".to_string())?;

    let proof = credential
        .get("proof")
        .ok_or_else(|| "credential.proof missing".to_string())?;
    let proof_value = proof
        .get("proofValue")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "proof.proofValue missing".to_string())?;

    let verifying_key = verifying_key_from_did(issuer)?;

    // Recreate the signed payload: credential without proof.
    let mut unsigned = credential.clone();
    if let Value::Object(map) = &mut unsigned {
        map.remove("proof");
    }
    let msg = serde_json::to_vec(&unsigned).map_err(|e| e.to_string())?;

    let (_base, sig_bytes) = multibase::decode(proof_value).map_err(|e| format!("proofValue decode failed: {e}"))?;
    let sig = Signature::from_slice(&sig_bytes).map_err(|e| e.to_string())?;
    Ok(verifying_key.verify(&msg, &sig).is_ok())
}
