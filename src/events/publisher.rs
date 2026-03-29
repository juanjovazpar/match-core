use crate::config::types::EventBusConfig;
use crate::events::bus::{create_event_bus, EventBus};
use crate::events::{queue::EventReceiver, serializer::serialize};
use crate::shared::event::EventEnvelope;

/// Event publisher loop
pub struct Publisher {
    rx: EventReceiver,
    bus: Box<dyn EventBus>,
}
impl Publisher {
    pub fn new(rx: EventReceiver, config: EventBusConfig) -> Self {
        let bus = create_event_bus(config);
        Self { rx, bus }
    }

    pub fn run(&self) {
        loop {
            let event = match self.rx.recv() {
                Ok(e) => e,
                Err(_) => break,
            };

            self.publish(event);
        }
    }

    fn publish(&self, event: EventEnvelope) {
        let subject = "match.events";
        let payload = serialize(&event).into_bytes();
        self.bus
            .publish(subject, payload)
            .unwrap_or_else(|e| panic!("failed to publish event to {subject}: {e}"));

        // TODO:
        // - retry
        // - backpressure
        // - batching
    }
}