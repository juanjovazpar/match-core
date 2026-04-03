use crate::shared::{
    types::{OrderId, Price, Quantity, Symbol, Timestamp, UserId, TimeInForce, Type, Side},
};

#[derive(Debug, Clone)]
pub struct Order {
    pub order_id: OrderId,
    pub user_id: UserId,
    pub symbol: Symbol,
    pub side: Side,
    pub order_type: Type,
    pub price: Price,
    pub quantity: Quantity,
    pub remaining: Quantity,
    pub timestamp: Timestamp,
    pub time_in_force: TimeInForce,
}
