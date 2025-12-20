use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use clap::Parser;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::net::SocketAddr;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use uuid::Uuid;

#[derive(Debug, Parser)]
struct Args {
    /// External Gateway URL for tool registration
    #[arg(long, env = "EXTERNAL_GATEWAY_URL", default_value = "http://127.0.0.1:8010")]
    external_gateway_url: String,

    /// Public base URL where the gateway can reach this plugin (must include scheme)
    #[arg(long, env = "PLUGIN_PUBLIC_URL", default_value = "http://127.0.0.1:9001")]
    plugin_public_url: String,

    /// Plugin bind address
    #[arg(long, env = "PLUGIN_BIND", default_value = "127.0.0.1:9001")]
    plugin_bind: String,
}

#[derive(Clone)]
struct PluginState {
    client: Client,
    plugin_id: String,
    external_gateway_url: String,
    plugin_public_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ToolSchema {
    pub name: String,
    pub description: String,
    pub plugin_url: String,
    pub endpoint: String,
    pub parameters: Value,
}

#[derive(Debug, Serialize)]
struct RegisterPayload {
    twin_id: Option<Value>,
    tool: ToolSchema,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let args = Args::parse();
    let plugin_id = format!("pagi-monitoring-plugin-{}", Uuid::new_v4());

    let state = PluginState {
        client: Client::new(),
        plugin_id: plugin_id.clone(),
        external_gateway_url: args.external_gateway_url.clone(),
        plugin_public_url: args.plugin_public_url.clone(),
    };

    register_tools(&state).await?;

    let app = Router::new()
        .route("/healthz", get(healthz))
        .route("/execute/:tool_name", post(execute_tool))
        .with_state(state)
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive());

    let addr: SocketAddr = args.plugin_bind.parse()?;
    tracing::info!(%addr, plugin_id = %plugin_id, "PAGI Monitoring Plugin listening");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

async fn register_tools(state: &PluginState) -> anyhow::Result<()> {
    let tools = vec![
        ToolSchema {
            name: "system_monitor".to_string(),
            description: "Monitor system health and performance metrics".to_string(),
            plugin_url: state.plugin_public_url.clone(),
            endpoint: "/execute/system_monitor".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "metric_type": {
                        "type": "string",
                        "enum": ["cpu", "memory", "disk", "network"],
                        "description": "Type of system metric to monitor"
                    },
                    "duration_seconds": {
                        "type": "integer",
                        "default": 60,
                        "description": "Duration to monitor in seconds"
                    }
                },
                "required": ["metric_type"]
            }),
        },
        ToolSchema {
            name: "log_analyzer".to_string(),
            description: "Analyze and filter log entries for patterns".to_string(),
            plugin_url: state.plugin_public_url.clone(),
            endpoint: "/execute/log_analyzer".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "log_level": {
                        "type": "string",
                        "enum": ["error", "warn", "info", "debug"],
                        "default": "info",
                        "description": "Log level to filter"
                    },
                    "pattern": {
                        "type": "string",
                        "description": "Substring pattern to search for"
                    },
                    "max_entries": {
                        "type": "integer",
                        "default": 100,
                        "description": "Maximum number of log entries to return"
                    }
                }
            }),
        },
        ToolSchema {
            name: "performance_profiler".to_string(),
            description: "Profile PAGI system performance".to_string(),
            plugin_url: state.plugin_public_url.clone(),
            endpoint: "/execute/performance_profiler".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "component": {
                        "type": "string",
                        "enum": ["all", "executive-engine", "inference-gateway", "context-builder"],
                        "default": "all",
                        "description": "Component to profile"
                    },
                    "depth": {
                        "type": "integer",
                        "default": 3,
                        "description": "Profiling depth level"
                    }
                }
            }),
        },
    ];

    let url = format!(
        "{}/register_tool",
        state.external_gateway_url.trim_end_matches('/')
    );

    for tool in tools {
        let payload = RegisterPayload {
            twin_id: None,
            tool: tool.clone(),
        };

        let resp = state.client.post(&url).json(&payload).send().await?;
        if !resp.status().is_success() {
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("tool registration failed for '{}': {}", tool.name, body);
        }

        tracing::info!(tool_name = %tool.name, plugin_id = %state.plugin_id, "Tool registered");
    }

    Ok(())
}

async fn healthz() -> (StatusCode, &'static str) {
    (StatusCode::OK, "ok")
}

#[derive(Debug, Serialize)]
struct ExecuteResponse {
    pub success: bool,
    pub result: Value,
    pub error: Option<String>,
}

async fn execute_tool(
    State(_state): State<PluginState>,
    Path(tool_name): Path<String>,
    Json(parameters): Json<Value>,
) -> Result<Json<ExecuteResponse>, (StatusCode, String)> {
    tracing::info!(tool_name = %tool_name, "Executing tool");

    let result = match tool_name.as_str() {
        "system_monitor" => execute_system_monitor(&parameters).await,
        "log_analyzer" => execute_log_analyzer(&parameters).await,
        "performance_profiler" => execute_performance_profiler(&parameters).await,
        _ => Err(format!("Unknown tool: {}", tool_name)),
    };

    match result {
        Ok(data) => Ok(Json(ExecuteResponse {
            success: true,
            result: data,
            error: None,
        })),
        Err(error) => Ok(Json(ExecuteResponse {
            success: false,
            result: Value::Null,
            error: Some(error),
        })),
    }
}

async fn execute_system_monitor(parameters: &Value) -> Result<Value, String> {
    let metric_type = parameters["metric_type"].as_str().ok_or("metric_type is required")?;
    let duration = parameters["duration_seconds"].as_u64().unwrap_or(60);

    // Simulated monitoring output
    Ok(json!({
        "metric_type": metric_type,
        "duration_seconds": duration,
        "timestamp": time::OffsetDateTime::now_utc(),
        "data": match metric_type {
            "cpu" => json!({"usage_percent": 45.2, "cores": 8, "load_average": [1.2, 1.5, 1.8]}),
            "memory" => json!({"total_gb": 16.0, "used_gb": 8.5, "available_gb": 7.5, "usage_percent": 53.1}),
            "disk" => json!({"total_gb": 500.0, "used_gb": 250.0, "free_gb": 250.0, "usage_percent": 50.0}),
            "network" => json!({"bytes_sent": 1024000, "bytes_received": 2048000, "packets_sent": 1024, "packets_received": 2048}),
            _ => json!({"error": "Unsupported metric type"}),
        }
    }))
}

async fn execute_log_analyzer(parameters: &Value) -> Result<Value, String> {
    let log_level = parameters["log_level"].as_str().unwrap_or("info");
    let pattern = parameters["pattern"].as_str();
    let max_entries = parameters["max_entries"].as_u64().unwrap_or(100);

    // Simulated log data
    let mock_logs = vec![
        json!({"timestamp": "2025-12-17T13:20:00Z", "level": "info", "message": "Service started successfully"}),
        json!({"timestamp": "2025-12-17T13:21:00Z", "level": "warn", "message": "High memory usage detected"}),
        json!({"timestamp": "2025-12-17T13:22:00Z", "level": "error", "message": "Database connection failed"}),
    ];

    let filtered_logs: Vec<Value> = mock_logs
        .into_iter()
        .filter(|log| {
            let matches_level = log["level"].as_str() == Some(log_level);
            let matches_pattern = pattern
                .map(|p| log["message"].as_str().unwrap_or("").contains(p))
                .unwrap_or(true);
            matches_level && matches_pattern
        })
        .take(max_entries as usize)
        .collect();

    Ok(json!({
        "log_level": log_level,
        "pattern": pattern,
        "total_entries": filtered_logs.len(),
        "logs": filtered_logs
    }))
}

async fn execute_performance_profiler(parameters: &Value) -> Result<Value, String> {
    let component = parameters["component"].as_str().unwrap_or("all");
    let depth = parameters["depth"].as_u64().unwrap_or(3);

    // Simulated performance profiling output
    Ok(json!({
        "component": component,
        "depth": depth,
        "timestamp": time::OffsetDateTime::now_utc(),
        "metrics": {
            "response_time_ms": 150,
            "throughput_rps": 1000,
            "error_rate_percent": 0.1,
            "cpu_usage_percent": 25.5,
            "memory_usage_mb": 512
        }
    }))
}

