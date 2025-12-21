pub mod events;
pub mod swarm;
pub mod types;

pub use events::{CoreEvent, EventEnvelope, EventType};
pub use swarm::{InstructionsField, Playbook, PlaybookInstructions, RefinementArtifact, ToolSchema};
pub use types::{TwinId, TwinState};

/// Common error type for cross-crate APIs.
///
/// Keep this intentionally lightweight to avoid pulling plugin-specific deps
/// (e.g. git2) into the core crates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[repr(u32)]
pub enum ErrorCode {
    ConfigInvalid = 1001,
    RedisError = 2002,
    PluginLoadFailed = 4001,
    PluginExecutionFailed = 4002,
    NetworkTimeout = 7001,
    Unknown = 9999,
}

#[derive(thiserror::Error, Debug)]
pub enum PagiError {
    #[error("Configuration error ({code:?}): {message}")]
    Config { code: ErrorCode, message: String },

    #[error("Redis error ({code:?}): {source}")]
    Redis { code: ErrorCode, source: redis::RedisError },

    #[error("Plugin error ({code:?}): {message}")]
    Plugin { code: ErrorCode, message: String },

    #[error("Network error ({code:?}): {source}")]
    Network { code: ErrorCode, source: reqwest::Error },

    #[error("IO error ({code:?}): {source}")]
    Io { code: ErrorCode, source: std::io::Error },

    #[error("Serialization error ({code:?}): {source}")]
    Serialization { code: ErrorCode, source: serde_json::Error },

    #[error("TOML error ({code:?}): {message}")]
    Toml { code: ErrorCode, message: String },

    #[error("Unknown error: {0}")]
    Unknown(String),
}

impl PagiError {
    pub fn code(&self) -> ErrorCode {
        match self {
            PagiError::Config { code, .. } => *code,
            PagiError::Redis { code, .. } => *code,
            PagiError::Plugin { code, .. } => *code,
            PagiError::Network { code, .. } => *code,
            PagiError::Io { code, .. } => *code,
            PagiError::Serialization { code, .. } => *code,
            PagiError::Toml { code, .. } => *code,
            PagiError::Unknown(_) => ErrorCode::Unknown,
        }
    }

    pub fn config(msg: impl Into<String>) -> Self {
        Self::Config {
            code: ErrorCode::ConfigInvalid,
            message: msg.into(),
        }
    }

    pub fn plugin_load(msg: impl Into<String>) -> Self {
        Self::Plugin {
            code: ErrorCode::PluginLoadFailed,
            message: msg.into(),
        }
    }

    pub fn plugin_exec(msg: impl Into<String>) -> Self {
        Self::Plugin {
            code: ErrorCode::PluginExecutionFailed,
            message: msg.into(),
        }
    }
}

impl From<std::io::Error> for PagiError {
    fn from(value: std::io::Error) -> Self {
        Self::Io {
            code: ErrorCode::Unknown,
            source: value,
        }
    }
}

impl From<reqwest::Error> for PagiError {
    fn from(value: reqwest::Error) -> Self {
        Self::Network {
            code: ErrorCode::Unknown,
            source: value,
        }
    }
}

impl From<serde_json::Error> for PagiError {
    fn from(value: serde_json::Error) -> Self {
        Self::Serialization {
            code: ErrorCode::Unknown,
            source: value,
        }
    }
}

impl From<toml::ser::Error> for PagiError {
    fn from(value: toml::ser::Error) -> Self {
        Self::Toml {
            code: ErrorCode::ConfigInvalid,
            message: value.to_string(),
        }
    }
}

impl From<toml::de::Error> for PagiError {
    fn from(value: toml::de::Error) -> Self {
        Self::Toml {
            code: ErrorCode::ConfigInvalid,
            message: value.to_string(),
        }
    }
}

/// Publish an event to the PAGI-EventRouter.
///
/// `EVENT_ROUTER_URL` can be either:
/// - a base URL (e.g. `http://localhost:8000`)
/// - the full publish endpoint (e.g. `http://localhost:8000/publish`)
pub async fn publish_event(envelope: EventEnvelope) -> Result<(), reqwest::Error> {
    let client = reqwest::Client::new();

    let mut url = std::env::var("EVENT_ROUTER_URL").unwrap_or_else(|_| "http://127.0.0.1:8000".to_string());
    if !url.ends_with("/publish") {
        url = format!("{}/publish", url.trim_end_matches('/'));
    }

    client.post(url).json(&envelope).send().await?.error_for_status()?;
    Ok(())
}
