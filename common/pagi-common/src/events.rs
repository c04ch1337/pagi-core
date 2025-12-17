use serde::{Deserialize, Serialize};
use serde_json::Value;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventType {
    GoalReceived,
    TwinRegistered,
    TwinStateUpdated,
    WorkingMemoryAppended,
    ContextBuilt,
    InferenceRequested,
    InferenceCompleted,
    PlanCreated,
    PlanGenerated,
    EmotionStateUpdated,
    ActionRequested,
}

impl EventType {
    pub fn as_str(&self) -> &'static str {
        match self {
            EventType::GoalReceived => "goal_received",
            EventType::TwinRegistered => "twin_registered",
            EventType::TwinStateUpdated => "twin_state_updated",
            EventType::WorkingMemoryAppended => "working_memory_appended",
            EventType::ContextBuilt => "context_built",
            EventType::InferenceRequested => "inference_requested",
            EventType::InferenceCompleted => "inference_completed",
            EventType::PlanCreated => "plan_created",
            EventType::PlanGenerated => "plan_generated",
            EventType::EmotionStateUpdated => "emotion_state_updated",
            EventType::ActionRequested => "action_requested",
        }
    }
}

/// Typed, core events for the AGI orchestration loop.
///
/// These are serialized into [`EventEnvelope::payload`](common/pagi-common/src/events.rs:73)
/// so services can share a common contract even when communicating via Kafka.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data", rename_all = "snake_case")]
pub enum CoreEvent {
    GoalReceived { goal: String },
    PlanGenerated { plan: String },
}

impl CoreEvent {
    pub fn event_type(&self) -> &'static str {
        match self {
            CoreEvent::GoalReceived { .. } => EventType::GoalReceived.as_str(),
            CoreEvent::PlanGenerated { .. } => EventType::PlanGenerated.as_str(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventEnvelope {
    pub id: Uuid,
    pub event_type: String,
    pub ts: OffsetDateTime,

    /// Optional correlation key.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub twin_id: Option<Uuid>,

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
            twin_id: None,
            subject: None,
            source: None,
            payload,
        }
    }

    pub fn new_core(twin_id: Uuid, core: CoreEvent) -> Self {
        let payload = serde_json::to_value(&core).unwrap_or(Value::Null);
        Self {
            id: Uuid::new_v4(),
            event_type: core.event_type().to_string(),
            ts: OffsetDateTime::now_utc(),
            twin_id: Some(twin_id),
            subject: None,
            source: None,
            payload,
        }
    }
}
