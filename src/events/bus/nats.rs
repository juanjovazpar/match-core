use tokio::runtime::Runtime;

use crate::events::bus::EventBus;

pub struct NatsBus {
    rt: Runtime,
    client: async_nats::Client,
}
impl NatsBus {
    pub fn new(url: &str) -> Self {
        let rt = Runtime::new().expect("failed to create Tokio runtime for NATS publisher");
        let client = rt
            .block_on(async_nats::connect(url))
            .unwrap_or_else(|e| panic!("failed to connect to NATS at {url}: {e}"));

        Self { rt, client }
    }
}
impl EventBus for NatsBus {
    fn publish(&self, subject: &str, payload: Vec<u8>) -> anyhow::Result<()> {
        let subject = subject.to_string();
        self.rt
            .block_on(self.client.publish(subject.clone(), payload.into()))
            .map_err(|e| anyhow::anyhow!("failed to publish NATS message to {subject}: {e}"))?;
        Ok(())
    }
}
