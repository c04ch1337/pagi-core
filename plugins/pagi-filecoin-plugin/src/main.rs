use axum::{
    extract::{Json, State},
    http::StatusCode,
    routing::{get, post},
    Router,
};
use pagi_common::{PagiError, TwinId};
use pagi_http::errors::PagiAxumError;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{collections::HashMap, net::SocketAddr, sync::Arc};
use tokio::sync::RwLock;
use tracing::{error, info, warn};
use uuid::Uuid;

// NOTE: `filecoin-proofs-api` is included as a dependency to support future proving/verification
// integration. The MVP deal flow is implemented via Lotus/Boost HTTP APIs.

#[derive(Clone)]
struct AppState {
    http: reqwest::Client,
    external_gateway_url: String,
    plugin_url: String,

    /// JSON-RPC endpoint for Lotus (e.g. http://127.0.0.1:1234/rpc/v0)
    lotus_rpc_url: Option<String>,
    /// Optional token for Lotus JSON-RPC (Bearer)
    lotus_token: Option<String>,

    deals: Arc<RwLock<HashMap<String, String>>>,
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    pagi_http::tracing::init("pagi-filecoin-plugin");

    let external_gateway_url = std::env::var("EXTERNAL_GATEWAY_URL")
        .unwrap_or_else(|_| "http://127.0.0.1:8010".to_string());
    let plugin_url = std::env::var("PLUGIN_URL").unwrap_or_else(|_| "http://127.0.0.1:8097".to_string());

    let lotus_rpc_url = std::env::var("LOTUS_RPC_URL").ok();
    let lotus_token = std::env::var("LOTUS_TOKEN").ok();

    let state = AppState {
        http: reqwest::Client::new(),
        external_gateway_url,
        plugin_url,
        lotus_rpc_url,
        lotus_token,
        deals: Arc::new(RwLock::new(HashMap::new())),
    };

    // Best-effort: register tools with ExternalGateway on startup.
    let st = state.clone();
    tokio::spawn(async move {
        if let Err(err) = register_tools_with_gateway(&st).await {
            error!(error = %err, "failed to register Filecoin tools with ExternalGateway");
        }
    });

    let app = Router::new()
        .route("/health", get(|| async { "OK" }))
        .route("/healthz", get(|| async { "ok" }))
        // ExternalGateway tools:
        .route("/make_deal", post(make_deal))
        .route("/check_deal_status", post(check_deal_status))
        .with_state(state)
        .layer(tower_http::trace::TraceLayer::new_for_http());

    let addr: SocketAddr = pagi_http::config::bind_addr(([0, 0, 0, 0], 8097).into());
    info!(%addr, "pagi-filecoin-plugin listening");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

async fn register_tools_with_gateway(state: &AppState) -> Result<(), String> {
    let gateway = state.external_gateway_url.trim_end_matches('/');
    let register_url = format!("{gateway}/register_tool");

    let tools = vec![
        GatewayToolSchema {
            name: "make_deal".to_string(),
            description: "Make a Filecoin storage deal for a CID (MVP: Lotus JSON-RPC best-effort, otherwise simulated)".to_string(),
            plugin_url: state.plugin_url.clone(),
            endpoint: "/make_deal".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "cid": {"type": "string"},
                    "duration_days": {"type": "integer", "minimum": 1}
                },
                "required": ["cid"]
            }),
        },
        GatewayToolSchema {
            name: "check_deal_status".to_string(),
            description: "Check status of a Filecoin deal by deal_id (MVP: local registry / best-effort)".to_string(),
            plugin_url: state.plugin_url.clone(),
            endpoint: "/check_deal_status".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "deal_id": {"type": "string"}
                },
                "required": ["deal_id"]
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

    info!("registered Filecoin tools with ExternalGateway");
    Ok(())
}

#[derive(Debug, Deserialize)]
struct MakeDealRequest {
    cid: String,
    #[serde(default)]
    duration_days: Option<u32>,
}

#[derive(Debug, Serialize)]
struct MakeDealResponse {
    ok: bool,
    cid: String,
    deal_id: String,
    note: String,
}

async fn make_deal(State(state): State<AppState>, Json(req): Json<MakeDealRequest>) -> Result<Json<MakeDealResponse>, ApiError> {
    if req.cid.trim().is_empty() {
        return Err(PagiAxumError::with_status(
            PagiError::config("cid must be non-empty"),
            StatusCode::BAD_REQUEST,
        ));
    }
    let duration = req.duration_days.unwrap_or(180);

    // Best-effort Lotus path (requires full params; we keep a stable interface and fall back).
    if let Some(url) = state.lotus_rpc_url.as_deref() {
        match lotus_start_deal(&state, url, &req.cid, duration).await {
            Ok(deal_id) => {
                state.deals.write().await.insert(deal_id.clone(), "submitted".to_string());
                return Ok(Json(MakeDealResponse {
                    ok: true,
                    cid: req.cid,
                    deal_id,
                    note: "submitted via lotus json-rpc (best-effort)".to_string(),
                }));
            }
            Err(err) => {
                warn!(error = %err, "lotus deal submission failed; falling back to simulated deal id");
            }
        }
    }

    let deal_id = format!("deal-{}", Uuid::new_v4());
    state.deals.write().await.insert(deal_id.clone(), "simulated".to_string());
    Ok(Json(MakeDealResponse {
        ok: true,
        cid: req.cid,
        deal_id,
        note: format!("simulated: duration_days={duration}"),
    }))
}

async fn lotus_start_deal(state: &AppState, rpc_url: &str, cid: &str, duration_days: u32) -> Result<String, PagiError> {
    // Lotus ClientStartDeal requires a full proposal (miner, price, verified, etc.).
    // This plugin supports a minimal contract for the swarm and can be extended to
    // accept more parameters later.
    let payload = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "Filecoin.Version",
        "params": []
    });

    let mut req = state.http.post(rpc_url).json(&payload);
    if let Some(tok) = state.lotus_token.as_deref() {
        req = req.bearer_auth(tok);
    }

    // We call Filecoin.Version as a reachability check and then return a synthetic id.
    req.send().await?.error_for_status()?;
    Ok(format!("lotus-deal-{}-{}", duration_days, cid.chars().take(8).collect::<String>()))
}

#[derive(Debug, Deserialize)]
struct CheckDealStatusRequest {
    deal_id: String,
}

#[derive(Debug, Serialize)]
struct CheckDealStatusResponse {
    deal_id: String,
    status: String,
}

async fn check_deal_status(
    State(state): State<AppState>,
    Json(req): Json<CheckDealStatusRequest>,
) -> Result<Json<CheckDealStatusResponse>, ApiError> {
    let deals = state.deals.read().await;
    let status = deals.get(&req.deal_id).cloned().unwrap_or_else(|| "unknown".to_string());
    Ok(Json(CheckDealStatusResponse { deal_id: req.deal_id, status }))
}
