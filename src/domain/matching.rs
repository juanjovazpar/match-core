use crate::shared::command::Command;
use crate::shared::event::*;

use super::order::Order;
use super::orderbook::OrderBook;

pub struct MatchingEngine {
    pub book: OrderBook,
}
impl MatchingEngine {
    pub fn new() -> Self {
        Self {
            book: OrderBook::new(),
        }
    }

    pub fn process(&mut self, _command_seq: u64, command: Command) -> Vec<Event> {
        match command {
            Command::NewOrder(cmd) => self.process_new_order(cmd),
            Command::CancelOrder(_cmd) => vec![], // simplificado
        }
    }

    fn process_new_order(&mut self, cmd: crate::shared::command::NewOrderCommand) -> Vec<Event> {
        let mut events = vec![];

        let order = Order {
            order_id: cmd.order_id.clone(),
            user_id: cmd.user_id,
            symbol: cmd.symbol.clone(),
            side: cmd.side,
            order_type: cmd.order_type,
            price: cmd.price,
            quantity: cmd.quantity,
            remaining: cmd.quantity,
            timestamp: cmd.timestamp,
            time_in_force: cmd.time_in_force,
        };

        // Simplificado: no matching real aún
        self.book.add_order(order.clone());

        events.push(Event::OrderAccepted(OrderAccepted {
            order_id: order.order_id,
            user_id: order.user_id,
            symbol: order.symbol,
        }));

        events
    }

    /// usado en recovery
    pub fn process_replay(&mut self, command: Command) {
        self.process(0, command);
    }
}
