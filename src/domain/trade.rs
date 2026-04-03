use crate::shared::types::{OrderId, Price, Quantity, Symbol, Timestamp, TradeId};

#[derive(Debug, Clone)]
pub struct Trade {
    pub trade_id: TradeId,
    pub symbol: Symbol,
    pub price: Price,
    pub quantity: Quantity,
    pub maker_order_id: OrderId,
    pub taker_order_id: OrderId,
    pub timestamp: Timestamp,
}
