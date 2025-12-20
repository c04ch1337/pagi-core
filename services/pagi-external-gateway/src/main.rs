mod auto_discover;
mod redis_registry;
mod shared_lib;
mod wasm_plugin;
mod wasm_component_plugin;

use axum::{
    extract::{Json, Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use pagi_common::TwinId;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{
    collections::HashMap,
    net::SocketAddr,
    path::{Path as StdPath, PathBuf},
    sync::Arc,
};
use tokio::sync::RwLock;
use tracing::info;
use uuid::Uuid;

use redis_registry::{load_all_tools, persist_tool};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSchema {
    pub name: String,
    pub description: String,
    pub plugin_url: String,
    pub endpoint: String,
    pub parameters: serde_json::Value,
}

#[derive(Clone)]
struct GatewayState {
    registry: Arc<RwLock<HashMap<Uuid, HashMap<String, ToolSchema>>>>,
    redis_client: redis::Client,
    http: reqwest::Client,
}

fn global_twin_id() -> TwinId {
    TwinId(Uuid::nil())
}

pub(crate) async fn upsert_tool(
    state: &GatewayState,
    twin_id: TwinId,
    tool: &ToolSchema,
) -> Result<(), redis::RedisError> {
    let twin_uuid = twin_id.0;
    {
        let mut reg = state.registry.write().await;
        reg.entry(twin_uuid)
            .or_default()
            .insert(tool.name.clone(), tool.clone());
    }

    // Persist (best-effort; surface errors to caller).
    let persist_twin = if twin_uuid == Uuid::nil() {
        None
    } else {
        Some(twin_uuid)
    };
    persist_tool(&state.redis_client, persist_twin, tool).await?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    pagi_http::tracing::init("pagi-external-gateway");

    let redis_url = std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
    let redis_client = redis::Client::open(redis_url.clone())?;

    // Load persisted tools into in-memory registry
    let loaded_registry = load_all_tools(&redis_client).await.unwrap_or_default();

    let state = GatewayState {
        registry: Arc::new(RwLock::new(loaded_registry)),
        redis_client,
        http: reqwest::Client::new(),
    };

    // Optional: auto-discovery from PLUGIN_DIR
    let auto_discover = std::env::var("AUTO_DISCOVER_PLUGINS")
        .unwrap_or_else(|_| "false".to_string())
        .to_lowercase();
    if auto_discover == "true" {
        let plugin_dir = std::env::var("PLUGIN_DIR").unwrap_or_else(|_| "/plugins".to_string());
        let state_clone = state.clone();
        tokio::spawn(async move {
            let plugin_dir_path = PathBuf::from(plugin_dir);
            if let Err(err) = auto_discover::spawn_plugin_watcher(plugin_dir_path, state_clone, true).await {
                tracing::error!(error = %err, "plugin watcher failed");
            }
        });
    }

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
    info!(%addr, %redis_url, "PAGI-ExternalGateway listening (Redis registry)");
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

    match upsert_tool(&state, twin_id, &tool).await {
        Ok(()) => {
            info!(tool_name = %tool.name, twin_id = ?twin_id, "Registered tool");
            StatusCode::OK.into_response()
        }
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("failed to register tool: {err}"),
        )
            .into_response(),
    }
}

async fn list_all_tools(State(state): State<GatewayState>) -> impl IntoResponse {
    let mut all_tools: Vec<ToolSchema> = Vec::new();

    let reg = state.registry.read().await;
    for tools in reg.values() {
        all_tools.extend(tools.values().cloned());
    }

    Json(json!({ "tools": all_tools })).into_response()
}

async fn list_tools_for_twin(
    Path(twin_uuid): Path<Uuid>,
    State(state): State<GatewayState>,
) -> impl IntoResponse {
    let twin_id = TwinId(twin_uuid);

    let reg = state.registry.read().await;
    let mut tools: Vec<ToolSchema> = Vec::new();

    if let Some(t) = reg.get(&twin_id.0) {
        tools.extend(t.values().cloned());
    }
    if let Some(global) = reg.get(&Uuid::nil()) {
        tools.extend(global.values().cloned());
    }

    Json(json!({ "twin_id": twin_id, "tools": tools })).into_response()
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
    let twin_uuid = payload.twin_id.0;
    let tool = {
        let reg = state.registry.read().await;
        reg.get(&twin_uuid)
            .and_then(|m| m.get(&tool_name))
            .or_else(|| reg.get(&Uuid::nil()).and_then(|m| m.get(&tool_name)))
            .cloned()
    };

    let Some(tool) = tool else {
        return (StatusCode::NOT_FOUND, format!("Tool '{tool_name}' not found")).into_response();
    };

    // Shared library execution path
    if let Some(lib_path) = tool.plugin_url.strip_prefix("sharedlib://") {
        return match shared_lib::execute_tool(StdPath::new(lib_path), &tool.endpoint, &payload.parameters) {
            Ok(result) => (StatusCode::OK, result).into_response(),
            Err(err) => (StatusCode::BAD_GATEWAY, err).into_response(),
        };
    }

    // WebAssembly execution path
    if let Some(wasm_path) = tool.plugin_url.strip_prefix("wasm://") {
        return match wasm_plugin::execute_tool(StdPath::new(wasm_path), &tool.endpoint, &payload.parameters) {
            Ok(result) => (StatusCode::OK, result).into_response(),
            Err(err) => (StatusCode::BAD_GATEWAY, err).into_response(),
        };
    }

    // WASI Component Model execution path
    if let Some(component_path) = tool
        .plugin_url
        .strip_prefix("wasm-component://")
        .or_else(|| tool.plugin_url.strip_prefix("component://"))
    {
        return match wasm_component_plugin::execute_tool(StdPath::new(component_path), &tool.endpoint, &payload.parameters) {
            Ok(result) => (StatusCode::OK, result).into_response(),
            Err(err) => (StatusCode::BAD_GATEWAY, err).into_response(),
        };
    }

    let base = tool.plugin_url.trim_end_matches('/');
    let endpoint = tool.endpoint.trim_start_matches('/');
    let url = format!("{base}/{endpoint}");

    match state.http.post(&url).json(&payload.parameters).send().await {
        Ok(resp) => {
            let status = resp.status();
            match resp.text().await {
                Ok(text) => {
                    if status.is_success() {
                        (StatusCode::OK, text).into_response()
                    } else {
                        (status, format!("Plugin returned {status}: {text}")).into_response()
                    }
                }
                Err(_) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Failed to read plugin response".to_string(),
                )
                    .into_response(),
            }
        }
        Err(e) => (StatusCode::BAD_GATEWAY, format!("Failed to reach plugin: {e}")).into_response(),
    }
}
