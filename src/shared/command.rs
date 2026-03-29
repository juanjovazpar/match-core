use serde::{Deserialize, Serialize};
use crate::shared::types::{CommandId, OrderId, Price, Quantity, Symbol, Timestamp, UserId};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Side {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OrderType {
    Limit,
    Market,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewOrderCommand {
    pub command_id: CommandId,
    pub order_id: OrderId,
    pub user_id: UserId,
    pub symbol: Symbol,
    pub side: Side,
    pub order_type: OrderType,
    pub price: Price,
    pub quantity: Quantity,
    pub timestamp: Timestamp,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CancelOrderCommand {
    pub command_id: CommandId,
    pub order_id: OrderId,
    pub user_id: UserId,
    pub symbol: Symbol,

    pub timestamp: Timestamp,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Command {
    NewOrder(NewOrderCommand),
    CancelOrder(CancelOrderCommand),
}