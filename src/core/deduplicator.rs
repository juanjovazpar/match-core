use crate::shared::command::Command;
use std::collections::HashSet;

/// Deduplicator component for incoming commands.
///
/// Maintains a set of processed command identifiers (idempotency keys).
/// It does help to prevent duplicate execution of the same command
/// by checking if the command has already been processed.
///
/// TODO:
/// This implementation uses an in-memory store that must be bounded.
/// A TTL to delete old entries must be added for production environments.
pub struct Deduplicator {
    seen: HashSet<String>,
}
impl Deduplicator {
    pub fn new() -> Self {
        Self {
            seen: HashSet::new(),
        }
    }

    pub fn exists(&mut self, command: &Command) -> bool {
        let id = Self::extract_id(command);

        if self.seen.contains(&id) {
            return true;
        }

        self.seen.insert(id);
        return false;
    }

    fn extract_id(command: &Command) -> String {
        match command {
            Command::NewOrder(c) => c.command_id.clone(),
            Command::CancelOrder(c) => c.command_id.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Deduplicator;
    use crate::shared::command::{
        CancelOrderCommand, Command, NewOrderCommand, Side, TimeInForce, Type,
    };

    fn new_order(command_id: &str) -> Command {
        Command::NewOrder(NewOrderCommand {
            command_id: command_id.into(),
            order_id: "1".into(),
            user_id: "u1".into(),
            symbol: "BTC-USD".into(),
            side: Side::Buy,
            order_type: Type::Limit,
            price: 1.0,
            quantity: 1.0,
            timestamp: 0,
            time_in_force: TimeInForce::GTC,
        })
    }

    fn cancel_order(command_id: &str) -> Command {
        Command::CancelOrder(CancelOrderCommand {
            command_id: command_id.into(),
            order_id: "1".into(),
            timestamp: 0,
        })
    }

    #[test]
    fn exists_first_time_is_false_then_true_for_same_id() {
        let mut d = Deduplicator::new();
        let cmd = new_order("id-a");

        assert!(!d.exists(&cmd));
        assert!(d.exists(&cmd));
    }

    #[test]
    fn exists_distinct_ids_are_independent() {
        let mut d = Deduplicator::new();
        let a = new_order("id-a");
        let b = new_order("id-b");

        assert!(!d.exists(&a));
        assert!(!d.exists(&b));
        assert!(d.exists(&a));
        assert!(d.exists(&b));
    }

    #[test]
    fn exists_dedupes_by_command_id_across_command_variants() {
        let mut d = Deduplicator::new();
        let new = new_order("same");
        let cancel = cancel_order("same");

        assert!(!d.exists(&new));
        assert!(d.exists(&cancel));
    }
}
