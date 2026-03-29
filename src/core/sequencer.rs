use crate::shared::types::{CommandSeq, EventSeq, TradeSeq};

/// Sequencer acts a clock to ensure consistent replay
/// 
/// It is responsible for generating monotonically increasing identifiers
/// used to guarantee ordering and determinism within the matching engine.
///
/// It produces:
/// - command_seq → total order of incoming commands
/// - event_seq   → total order of emitted events
/// - trade_seq   → ordered sequence of trades
/// 
pub struct Sequencer {
    pub command_seq: CommandSeq,
    pub event_seq: EventSeq,
    pub trade_seq: TradeSeq,
}
impl Sequencer {
    pub fn new() -> Self {
        Self {
            command_seq: 0,
            event_seq: 0,
            trade_seq: 0,
        }
    }

    pub fn next_command_seq(&mut self) -> CommandSeq {
        self.command_seq += 1;
        self.command_seq
    }

    pub fn next_event_seq(&mut self) -> EventSeq {
        self.event_seq += 1;
        self.event_seq
    }

    pub fn next_trade_seq(&mut self) -> TradeSeq {
        self.trade_seq += 1;
        self.trade_seq
    }
}

#[cfg(test)]
mod tests {
    use super::Sequencer;

    #[test]
    fn new_starts_at_zero() {
        let s = Sequencer::new();
        assert_eq!(s.command_seq, 0);
        assert_eq!(s.event_seq, 0);
        assert_eq!(s.trade_seq, 0);
    }

    #[test]
    fn next_command_seq_is_monotonic() {
        let mut s = Sequencer::new();
        assert_eq!(s.next_command_seq(), 1);
        assert_eq!(s.next_command_seq(), 2);
        assert_eq!(s.command_seq, 2);
    }

    #[test]
    fn next_event_seq_is_monotonic() {
        let mut s = Sequencer::new();
        assert_eq!(s.next_event_seq(), 1);
        assert_eq!(s.next_event_seq(), 2);
        assert_eq!(s.event_seq, 2);
    }

    #[test]
    fn next_trade_seq_is_monotonic() {
        let mut s = Sequencer::new();
        assert_eq!(s.next_trade_seq(), 1);
        assert_eq!(s.next_trade_seq(), 2);
        assert_eq!(s.trade_seq, 2);
    }

    #[test]
    fn sequences_are_independent() {
        let mut s = Sequencer::new();
        assert_eq!(s.next_command_seq(), 1);
        assert_eq!(s.next_event_seq(), 1);
        assert_eq!(s.next_trade_seq(), 1);
        assert_eq!(s.next_command_seq(), 2);
        assert_eq!(s.event_seq, 1);
        assert_eq!(s.trade_seq, 1);
    }
}