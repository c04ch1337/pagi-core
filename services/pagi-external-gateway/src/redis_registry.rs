use redis::{AsyncCommands, RedisError};
use std::collections::HashMap;
use tracing::info;
use uuid::Uuid;

use crate::ToolSchema;

/// Redis key patterns
const GLOBAL_TOOLS_KEY: &str = "pagi:tools:global";
const TWIN_TOOLS_PREFIX: &str = "pagi:tools:twin:";

/// Helper to generate twin-specific key
fn twin_key(twin_id: &Uuid) -> String {
    format!("{TWIN_TOOLS_PREFIX}{twin_id}")
}

/// Load all tools from Redis into an in-memory registry.
///
/// The in-memory registry uses:
/// - `Uuid::nil()` for global tools
/// - actual `Uuid` for twin-specific tools
pub async fn load_all_tools(
    client: &redis::Client,
) -> Result<HashMap<Uuid, HashMap<String, ToolSchema>>, RedisError> {
    let mut con = client.get_multiplexed_tokio_connection().await?;
    let mut registry: HashMap<Uuid, HashMap<String, ToolSchema>> = HashMap::new();

    // Load global tools
    let global_keys: Vec<String> = con.hkeys(GLOBAL_TOOLS_KEY).await?;
    for key in global_keys {
        let tool_json: String = con.hget(GLOBAL_TOOLS_KEY, &key).await?;
        if let Ok(tool) = serde_json::from_str::<ToolSchema>(&tool_json) {
            registry
                .entry(Uuid::nil())
                .or_default()
                .insert(key, tool);
        }
    }

    // Load twin-specific tools
    let twin_keys: Vec<String> = con.keys(format!("{TWIN_TOOLS_PREFIX}*")).await?;
    for key in twin_keys {
        let twin_id_str = key.strip_prefix(TWIN_TOOLS_PREFIX).unwrap_or("");
        if let Ok(twin_id) = Uuid::parse_str(twin_id_str) {
            let tool_names: Vec<String> = con.hkeys(&key).await?;
            for name in tool_names {
                let tool_json: String = con.hget(&key, &name).await?;
                if let Ok(tool) = serde_json::from_str::<ToolSchema>(&tool_json) {
                    registry.entry(twin_id).or_default().insert(name, tool);
                }
            }
        }
    }

    info!(groups = registry.len(), "Loaded tool groups from Redis");
    Ok(registry)
}

/// Persist a tool registration to Redis.
///
/// - `twin_id = None` => global tool
/// - `twin_id = Some(uuid)` => twin-specific tool
pub async fn persist_tool(
    client: &redis::Client,
    twin_id: Option<Uuid>,
    tool: &ToolSchema,
) -> Result<(), RedisError> {
    let mut con = client.get_multiplexed_tokio_connection().await?;
    let tool_json = serde_json::to_string(tool).map_err(|e| {
        redis::RedisError::from((
            redis::ErrorKind::TypeError,
            "Serialization error",
            e.to_string(),
        ))
    })?;

    let key = match twin_id {
        Some(id) => twin_key(&id),
        None => GLOBAL_TOOLS_KEY.to_string(),
    };

    con.hset::<_, _, _, ()>(&key, &tool.name, tool_json).await?;
    info!(tool_name = %tool.name, redis_key = %key, "Persisted tool to Redis");
    Ok(())
}

/// Remove a tool (optional cleanup)
#[allow(dead_code)]
pub async fn remove_tool(
    client: &redis::Client,
    twin_id: Option<Uuid>,
    tool_name: &str,
) -> Result<(), RedisError> {
    let mut con = client.get_multiplexed_tokio_connection().await?;
    let key = match twin_id {
        Some(id) => twin_key(&id),
        None => GLOBAL_TOOLS_KEY.to_string(),
    };

    con.hdel::<_, _, ()>(&key, tool_name).await?;
    Ok(())
}
