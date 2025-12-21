use axum::{
    extract::{Json, State},
    routing::{get, post},
    Router,
};
use base64::Engine;
use pagi_common::{PagiError, TwinId};
use pagi_http::errors::PagiAxumError;
use reqwest::multipart;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::net::SocketAddr;
use tracing::{error, info};

#[cfg(feature = "embedded-ipfs")]
use rust_ipfs::{Ipfs, IpfsOptions, Multiaddr, StoragePath};

#[cfg(feature = "embedded-ipfs")]
use std::sync::Arc;

#[cfg(feature = "embedded-ipfs")]
use tracing::warn;

#[derive(Clone)]
struct AppState {
    http: reqwest::Client,
    external_gateway_url: String,
    plugin_url: String,

    /// IPFS daemon HTTP API base (e.g. http://127.0.0.1:5001)
    ipfs_api_url: String,

    /// Optional embedded IPFS node (requires feature `embedded-ipfs`).
    #[cfg(feature = "embedded-ipfs")]
    ipfs: Option<Arc<Ipfs>>,
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

type ApiError = PagiAxumError;

fn to_api_err(err: PagiError) -> ApiError {
    err.into()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    pagi_http::tracing::init("pagi-ipfs-plugin");

    let external_gateway_url = std::env::var("EXTERNAL_GATEWAY_URL")
        .unwrap_or_else(|_| "http://127.0.0.1:8010".to_string());
    let plugin_url = std::env::var("PLUGIN_URL").unwrap_or_else(|_| "http://127.0.0.1:8096".to_string());

    let ipfs_api_url = std::env::var("IPFS_API_URL").unwrap_or_else(|_| "http://127.0.0.1:5001".to_string());

    let state = AppState {
        http: reqwest::Client::new(),
        external_gateway_url,
        plugin_url,
        ipfs_api_url,

        #[cfg(feature = "embedded-ipfs")]
        ipfs: {
            let mode = std::env::var("IPFS_MODE").unwrap_or_else(|_| "http".to_string());
            if mode.to_lowercase() == "embedded" {
                match init_ipfs().await {
                    Ok(i) => Some(Arc::new(i)),
                    Err(err) => {
                        warn!(error = %err, "embedded IPFS init failed; continuing in http-only mode");
                        None
                    }
                }
            } else {
                None
            }
        },
    };

    // Best-effort: register tools with ExternalGateway on startup.
    let st = state.clone();
    tokio::spawn(async move {
        if let Err(err) = register_tools_with_gateway(&st).await {
            error!(error = %err, "failed to register IPFS tools with ExternalGateway");
        }
    });

    let app = Router::new()
        .route("/health", get(|| async { "OK" }))
        .route("/healthz", get(|| async { "ok" }))
        // ExternalGateway tools:
        .route("/upload_file", post(upload_file))
        .route("/retrieve_file", post(retrieve_file))
        .with_state(state)
        .layer(tower_http::trace::TraceLayer::new_for_http());

    let addr: SocketAddr = pagi_http::config::bind_addr(([0, 0, 0, 0], 8096).into());
    info!(%addr, "pagi-ipfs-plugin listening");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

#[cfg(feature = "embedded-ipfs")]
async fn init_ipfs() -> Result<Ipfs, PagiError> {
    let mut opts = IpfsOptions::default();

    // NOTE: `rust-ipfs` defaults to an in-memory repo; for long-running relay nodes you almost
    // certainly want persistence.
    if let Ok(path) = std::env::var("IPFS_REPO_PATH") {
        if !path.trim().is_empty() {
            opts.ipfs_path = StoragePath::Path(path.into());
        }
    }

    // Enable peer discovery + routing by default.
    opts.mdns = env_bool("IPFS_MDNS", true);
    opts.disable_kad = !env_bool("IPFS_KAD", true);
    opts.pubsub_config = env_bool("IPFS_PUBSUB", true).then(|| Default::default());

    // Circuit relay v2: enable client, optionally enable server.
    opts.relay = env_bool("IPFS_RELAY", true);
    opts.relay_server = env_bool("IPFS_RELAY_SERVER", false);

    // Listening addresses: comma-separated multiaddrs.
    // Example: /ip4/0.0.0.0/tcp/4001,/ip6/::/tcp/4001
    opts.listening_addrs = match std::env::var("IPFS_LISTEN_ADDRS") {
        Ok(v) if !v.trim().is_empty() => parse_multiaddrs(&v),
        _ => vec!["/ip4/0.0.0.0/tcp/4001".parse().expect("valid multiaddr")],
    };

    // Bootstrap peers (recommended for relays): comma-separated multiaddrs.
    if let Ok(v) = std::env::var("IPFS_BOOTSTRAP") {
        let peers = parse_multiaddrs(&v);
        if !peers.is_empty() {
            opts.bootstrap = peers;
        }
    }

    Ipfs::new(opts)
        .await
        .map_err(|e| PagiError::plugin_exec(format!("rust-ipfs init failed: {e}")))
}

#[cfg(feature = "embedded-ipfs")]
fn env_bool(key: &str, default: bool) -> bool {
    match std::env::var(key) {
        Ok(v) => matches!(v.trim().to_ascii_lowercase().as_str(), "1" | "true" | "yes" | "on"),
        Err(_) => default,
    }
}

#[cfg(feature = "embedded-ipfs")]
fn parse_multiaddrs(raw: &str) -> Vec<Multiaddr> {
    raw.split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .filter_map(|s| s.parse().ok())
        .collect()
}

async fn register_tools_with_gateway(state: &AppState) -> Result<(), String> {
    let gateway = state.external_gateway_url.trim_end_matches('/');
    let register_url = format!("{gateway}/register_tool");

    let tools = vec![
        GatewayToolSchema {
            name: "upload_file".to_string(),
            description: "Upload a file to embedded IPFS and return a CID".to_string(),
            plugin_url: state.plugin_url.clone(),
            endpoint: "/upload_file".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "file_path": {"type": "string"}
                },
                "required": ["file_path"]
            }),
        },
        GatewayToolSchema {
            name: "retrieve_file".to_string(),
            description: "Retrieve file bytes from embedded IPFS by CID (returns base64)".to_string(),
            plugin_url: state.plugin_url.clone(),
            endpoint: "/retrieve_file".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "cid": {"type": "string"}
                },
                "required": ["cid"]
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

    info!("registered IPFS tools with ExternalGateway");
    Ok(())
}

#[derive(Debug, Deserialize)]
struct UploadRequest {
    file_path: String,
}

#[derive(Debug, Serialize)]
struct UploadResponse {
    cid: String,
}

async fn upload_file(State(state): State<AppState>, Json(req): Json<UploadRequest>) -> Result<Json<UploadResponse>, ApiError> {
    // Prefer embedded mode if available.
    #[cfg(feature = "embedded-ipfs")]
    if let Some(ipfs) = state.ipfs.as_ref() {
        let bytes = tokio::fs::read(&req.file_path).await.map_err(|e| to_api_err(e.into()))?;
        let cid = ipfs
            .add(bytes.into())
            .await
            .map_err(|e| to_api_err(PagiError::plugin_exec(format!("ipfs add failed: {e}"))))?;
        return Ok(Json(UploadResponse { cid: cid.to_string() }));
    }

    // Fallback: IPFS HTTP API.
    let cid = upload_to_ipfs_http(&state.http, &state.ipfs_api_url, &req.file_path)
        .await
        .map_err(to_api_err)?;
    Ok(Json(UploadResponse { cid }))
}

#[derive(Debug, Deserialize)]
struct RetrieveRequest {
    cid: String,
}

#[derive(Debug, Serialize)]
struct RetrieveResponse {
    cid: String,
    data_b64: String,
}

async fn retrieve_file(State(state): State<AppState>, Json(req): Json<RetrieveRequest>) -> Result<Json<RetrieveResponse>, ApiError> {
    #[cfg(feature = "embedded-ipfs")]
    if let Some(ipfs) = state.ipfs.as_ref() {
        let cid = req
            .cid
            .parse()
            .map_err(|e| to_api_err(PagiError::plugin_exec(format!("invalid cid: {e}"))))?;
        let data = ipfs
            .get(&cid)
            .await
            .map_err(|e| to_api_err(PagiError::plugin_exec(format!("ipfs get failed: {e}"))))?;
        let b64 = base64::engine::general_purpose::STANDARD.encode(data.to_vec());
        return Ok(Json(RetrieveResponse { cid: req.cid, data_b64: b64 }));
    }

    let bytes = retrieve_from_ipfs_http(&state.http, &state.ipfs_api_url, &req.cid)
        .await
        .map_err(to_api_err)?;
    let b64 = base64::engine::general_purpose::STANDARD.encode(bytes);
    Ok(Json(RetrieveResponse { cid: req.cid, data_b64: b64 }))
}

#[derive(Debug, Deserialize)]
struct IpfsAddLine {
    #[serde(rename = "Hash")]
    hash: String,
}

async fn upload_to_ipfs_http(http: &reqwest::Client, ipfs_api_url: &str, file_path: &str) -> Result<String, PagiError> {
    let bytes = tokio::fs::read(file_path).await?;
    let file_name = std::path::Path::new(file_path)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("artifact.bin")
        .to_string();

    let part = multipart::Part::bytes(bytes).file_name(file_name);
    let form = multipart::Form::new().part("file", part);

    let base = ipfs_api_url.trim_end_matches('/');
    let url = format!("{base}/api/v0/add?pin=true");
    let text = http
        .post(url)
        .multipart(form)
        .send()
        .await?
        .error_for_status()?
        .text()
        .await?;

    let last = text.lines().last().unwrap_or("");
    let parsed: IpfsAddLine = serde_json::from_str(last)
        .map_err(|e| PagiError::plugin_exec(format!("invalid IPFS add response: {e}; raw={text}")))?;
    Ok(parsed.hash)
}

async fn retrieve_from_ipfs_http(http: &reqwest::Client, ipfs_api_url: &str, cid: &str) -> Result<Vec<u8>, PagiError> {
    let base = ipfs_api_url.trim_end_matches('/');
    let url = format!("{base}/api/v0/cat?arg={cid}");
    let bytes = http.post(url).send().await?.error_for_status()?.bytes().await?;
    Ok(bytes.to_vec())
}
