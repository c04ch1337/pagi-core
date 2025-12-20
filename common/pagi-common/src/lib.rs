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
#[derive(Debug, thiserror::Error)]
pub enum PagiError {
    #[error("{0}")]
    Message(String),
}

impl From<std::io::Error> for PagiError {
    fn from(value: std::io::Error) -> Self {
        Self::Message(value.to_string())
    }
}

impl From<reqwest::Error> for PagiError {
    fn from(value: reqwest::Error) -> Self {
        Self::Message(value.to_string())
    }
}

impl From<serde_json::Error> for PagiError {
    fn from(value: serde_json::Error) -> Self {
        Self::Message(value.to_string())
    }
}

impl From<toml::ser::Error> for PagiError {
    fn from(value: toml::ser::Error) -> Self {
        Self::Message(value.to_string())
    }
}

impl From<toml::de::Error> for PagiError {
    fn from(value: toml::de::Error) -> Self {
        Self::Message(value.to_string())
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
