pub mod events;
pub mod types;

pub use events::{CoreEvent, EventEnvelope, EventType};
pub use types::{TwinId, TwinState};

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
