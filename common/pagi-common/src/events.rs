use serde::{Deserialize, Serialize};
use serde_json::Value;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventType {
    TwinRegistered,
    TwinStateUpdated,
    WorkingMemoryAppended,
    ContextBuilt,
    InferenceRequested,
    InferenceCompleted,
    PlanCreated,
    EmotionStateUpdated,
    ActionRequested,
}

impl EventType {
    pub fn as_str(&self) -> &'static str {
        match self {
            EventType::TwinRegistered => "twin_registered",
            EventType::TwinStateUpdated => "twin_state_updated",
            EventType::WorkingMemoryAppended => "working_memory_appended",
            EventType::ContextBuilt => "context_built",
            EventType::InferenceRequested => "inference_requested",
            EventType::InferenceCompleted => "inference_completed",
            EventType::PlanCreated => "plan_created",
            EventType::EmotionStateUpdated => "emotion_state_updated",
            EventType::ActionRequested => "action_requested",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventEnvelope {
    pub id: Uuid,
    pub event_type: String,
    pub ts: OffsetDateTime,

    /// Optional routing metadata ("subject") and origin identity ("source").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subject: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,

    pub payload: Value,
}

impl EventEnvelope {
    pub fn new(event_type: EventType, payload: Value) -> Self {
        Self {
            id: Uuid::new_v4(),
            event_type: event_type.as_str().to_string(),
            ts: OffsetDateTime::now_utc(),
            subject: None,
            source: None,
            payload,
        }
    }
}

