use crate::config::types::EventBusConfig;

mod kafka;
mod nats;

pub use kafka::KafkaBus;
pub use nats::NatsBus;

pub trait EventBus: Send + Sync {
    fn publish(&self, subject: &str, payload: Vec<u8>) -> anyhow::Result<()>;
}

pub fn create_event_bus(config: EventBusConfig) -> Box<dyn EventBus> {
    match config.r#type.as_str() {
        "nats" => Box::new(NatsBus::new(&config.url)),
        "kafka" => Box::new(KafkaBus::new(config.url)),
        other => panic!("unsupported event bus type: {other} (expected 'nats' or 'kafka')"),
    }
}

