use crate::events::bus::EventBus;

pub struct KafkaBus {
    url: String,
}

impl KafkaBus {
    pub fn new(url: String) -> Self {
        Self { url }
    }
}

impl EventBus for KafkaBus {
    fn publish(&self, subject: &str, payload: Vec<u8>) -> anyhow::Result<()> {
        // TODO: Implement Kafka event bus
        println!(
            "📡 (kafka placeholder @ {}) Publishing to {}: {}",
            self.url,
            subject,
            String::from_utf8_lossy(&payload)
        );
        Ok(())
    }
}
