use pagi_common::{EventEnvelope, EventType};
use serde_json::json;

#[test]
fn event_envelope_new_sets_required_fields() {
    let ev = EventEnvelope::new(EventType::TwinRegistered, json!({"twin_id": "abc"}));

    assert!(!ev.id.is_nil());
    assert_eq!(ev.event_type, "twin_registered");
    assert!(ev.payload.get("twin_id").is_some());
}

