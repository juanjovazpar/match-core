pub use crate::shared::types::{Side, TimeInForce, Type};
use crate::shared::types::{CommandId, OrderId, Price, Quantity, Symbol, Timestamp, UserId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewOrderCommand {
    pub command_id: CommandId,
    pub order_id: OrderId,
    pub user_id: UserId,
    pub symbol: Symbol,
    pub side: Side,
    pub order_type: Type,
    pub price: Price,
    pub quantity: Quantity,
    pub timestamp: Timestamp,
    pub time_in_force: TimeInForce,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CancelOrderCommand {
    pub command_id: CommandId,
    pub order_id: OrderId,
    pub timestamp: Timestamp,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Command {
    NewOrder(NewOrderCommand),
    CancelOrder(CancelOrderCommand),
}
