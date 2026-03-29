
use crate::shared::{command::{OrderType, Side}, types::{OrderId, Price, Quantity, Symbol, Timestamp, UserId}};

#[derive(Debug, Clone)]
pub struct Order {
    pub order_id: OrderId,
    pub user_id: UserId,
    pub symbol: Symbol,
    pub side: Side,
    pub order_type: OrderType,
    pub price: Price,
    pub quantity: Quantity,
    pub remaining: Quantity,
    pub timestamp: Timestamp,
}