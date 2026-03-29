use crate::shared::types::CommandSeq;
use crate::shared::{command::Command, event::EventEnvelope};
use std::fs::{OpenOptions};
use std::io::Write;

/// LogWriter responsible for durability of the engine.
///
/// Ensures that every command and resulting events are persisted to disk
/// before being applied to the in-memory state. It is append-only 
/// and strictly ordered by sequence numbers.
/// TODO: Optimize file writing to make it faster
pub struct LogWriter {
    file: std::fs::File,
    segment_size: String,
}
impl LogWriter {
    pub fn new(path: &str, segment_size: String) -> Self {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .unwrap();

        Self { file, segment_size}
    }

    pub fn write_command(&mut self, seq: CommandSeq, cmd: &Command) {
        let line = format!("CMD|{}|{:?}\n", seq, cmd);
        self.append_synced(line.as_bytes());
    }

    pub fn write_event(&mut self, event: &EventEnvelope) {
        let line = format!("EVT|{}|{:?}\n", event.event_seq, event);
        self.append_synced(line.as_bytes());
    }

    fn append_synced(&mut self, data: &[u8]) {
        self.file.write_all(data).unwrap();
        self.file.sync_all().unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::LogWriter;
    use crate::shared::command::{Command, NewOrderCommand, OrderType, Side};
    use crate::shared::event::{Event, EventEnvelope, OrderAccepted};
    use std::fs;

    fn sample_command() -> Command {
        Command::NewOrder(NewOrderCommand {
            command_id: "cmd-1".into(),
            order_id: 42,
            user_id: "u1".into(),
            symbol: "BTC-USD".into(),
            side: Side::Buy,
            order_type: OrderType::Limit,
            price: 1.0,
            quantity: 2.0,
            timestamp: 99,
        })
    }

    fn sample_event() -> EventEnvelope {
        EventEnvelope {
            event_seq: 3,
            command_seq: 7,
            timestamp: 100,
            event: Event::OrderAccepted(OrderAccepted {
                order_id: 42,
                user_id: "u1".into(),
                symbol: "BTC-USD".into(),
            }),
        }
    }

    #[test]
    fn write_command_appends_cmd_line() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("wal.log");
        let path_str = path.to_str().unwrap();

        {
            let mut w = LogWriter::new(path_str, "1MB".to_string());
            w.write_command(1, &sample_command());
        }

        let contents = fs::read_to_string(&path).unwrap();
        assert!(
            contents.starts_with("CMD|1|"),
            "expected CMD line prefix, got: {contents:?}"
        );
        assert!(contents.contains("NewOrder"));
        assert!(contents.ends_with("\n"));
    }

    #[test]
    fn write_event_appends_evt_line() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("wal.log");
        let path_str = path.to_str().unwrap();

        {
            let mut w = LogWriter::new(path_str, "1MB".to_string());
            w.write_event(&sample_event());
        }

        let contents = fs::read_to_string(&path).unwrap();
        assert!(
            contents.starts_with("EVT|3|"),
            "expected EVT line prefix, got: {contents:?}"
        );
        assert!(contents.contains("OrderAccepted"));
        assert!(contents.ends_with("\n"));
    }

    #[test]
    fn writes_are_appended_in_order() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("wal.log");
        let path_str = path.to_str().unwrap();

        {
            let mut w = LogWriter::new(path_str, "1MB".to_string());
            w.write_command(10, &sample_command());
            w.write_event(&sample_event());
        }

        let contents = fs::read_to_string(&path).unwrap();
        let lines: Vec<&str> = contents.lines().collect();
        assert_eq!(lines.len(), 2);
        assert!(lines[0].starts_with("CMD|10|"));
        assert!(lines[1].starts_with("EVT|3|"));
    }
}