use axum::{
    extract::{Json, Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use once_cell::sync::Lazy;
use pagi_common::TwinId;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{collections::HashMap, net::SocketAddr, sync::Arc};
use tokio::sync::RwLock;
use tracing::info;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSchema {
    pub name: String,
    pub description: String,
    pub plugin_url: String,
    pub endpoint: String,
    pub parameters: serde_json::Value,
}

type ToolRegistry = HashMap<String, ToolSchema>;
type SharedRegistry = Arc<RwLock<HashMap<TwinId, ToolRegistry>>>;

static REGISTRY: Lazy<SharedRegistry> = Lazy::new(|| Arc::new(RwLock::new(HashMap::new())));

#[derive(Clone)]
struct GatewayState {
    registry: SharedRegistry,
    http: reqwest::Client,
}

fn global_twin_id() -> TwinId {
    TwinId(Uuid::nil())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    pagi_http::tracing::init("pagi-external-gateway");

    let state = GatewayState {
        registry: REGISTRY.clone(),
        http: reqwest::Client::new(),
    };

    let app = Router::new()
        .route("/health", get(|| async { "OK" }))
        .route("/healthz", get(|| async { "OK" }))
        .route("/register_tool", post(register_tool))
        .route("/tools", get(list_all_tools))
        .route("/tools/:twin_id", get(list_tools_for_twin))
        .route("/execute/:tool_name", post(execute_tool))
        .with_state(state);

    let addr: SocketAddr = pagi_http::config::bind_addr(([0, 0, 0, 0], 8010).into());
    let listener = tokio::net::TcpListener::bind(addr).await?;
    info!(%addr, "PAGI-ExternalGateway listening (dynamic tool registry)");
    axum::serve(listener, app).await?;
    Ok(())
}

#[derive(Deserialize)]
struct RegisterPayload {
    twin_id: Option<TwinId>,
    tool: ToolSchema,
}

async fn register_tool(
    State(state): State<GatewayState>,
    Json(payload): Json<RegisterPayload>,
) -> impl IntoResponse {
    let twin_id = payload.twin_id.unwrap_or_else(global_twin_id);
    let tool = payload.tool;

    let mut registry = state.registry.write().await;
    let tools = registry.entry(twin_id).or_default();
    tools.insert(tool.name.clone(), tool.clone());

    info!(tool_name = %tool.name, twin_id = ?twin_id, "Registered tool");
    StatusCode::OK
}

async fn list_all_tools(State(state): State<GatewayState>) -> Json<serde_json::Value> {
    let registry = state.registry.read().await;
    let all: Vec<ToolSchema> = registry
        .values()
        .flat_map(|m| m.values().cloned())
        .collect();

    Json(json!({ "tools": all }))
}

async fn list_tools_for_twin(
    Path(twin_uuid): Path<Uuid>,
    State(state): State<GatewayState>,
) -> Json<serde_json::Value> {
    let twin_id = TwinId(twin_uuid);

    let registry = state.registry.read().await;
    let tools = registry.get(&twin_id).cloned().unwrap_or_default();
    let tools_vec: Vec<ToolSchema> = tools.values().cloned().collect();

    Json(json!({ "twin_id": twin_id, "tools": tools_vec }))
}

#[derive(Deserialize)]
struct ExecutePayload {
    twin_id: TwinId,
    parameters: serde_json::Value,
}

async fn execute_tool(
    Path(tool_name): Path<String>,
    State(state): State<GatewayState>,
    Json(payload): Json<ExecutePayload>,
) -> impl IntoResponse {
    let registry = state.registry.read().await;

    let tool = registry
        .get(&payload.twin_id)
        .and_then(|m| m.get(&tool_name))
        .or_else(|| registry.get(&global_twin_id()).and_then(|m| m.get(&tool_name)));

    let Some(tool) = tool else {
        return (StatusCode::NOT_FOUND, format!("Tool '{tool_name}' not found"));
    };

    let base = tool.plugin_url.trim_end_matches('/');
    let endpoint = tool.endpoint.trim_start_matches('/');
    let url = format!("{base}/{endpoint}");

    match state.http.post(&url).json(&payload.parameters).send().await {
        Ok(resp) => {
            let status = resp.status();
            match resp.text().await {
                Ok(text) => {
                    if status.is_success() {
                        (StatusCode::OK, text)
                    } else {
                        (status, format!("Plugin returned {status}: {text}"))
                    }
                }
                Err(_) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Failed to read plugin response".into(),
                ),
            }
        }
        Err(e) => (StatusCode::BAD_GATEWAY, format!("Failed to reach plugin: {e}")),
    }
}

