use crate::shared::event::EventEnvelope;

pub fn serialize(event: &EventEnvelope) -> String {
    serde_json::to_string(event).unwrap()
}
