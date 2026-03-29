use crate::{domain::matching::MatchingEngine, shared::{command::Command, event::EventEnvelope}};

pub struct Replayer;

impl Replayer {
    pub fn replay(
        engine: &mut MatchingEngine,
        wal_entries: Vec<String>,
    ) {
        for entry in wal_entries {
            if entry.starts_with("CMD") {
                if let Some(cmd) = Self::parse_command(&entry) {
                    engine.process_replay(cmd);
                }
            }
        }
    }

    fn parse_command(line: &str) -> Option<Command> {
        // ⚠️ simplificado (en prod usar serde/bincode)
        // ejemplo: CMD|1|NewOrder(...)
        None
    }

    #[allow(dead_code)]
    fn parse_event(line: &str) -> Option<EventEnvelope> {
        // opcional si decides reusar eventos
        None
    }
}