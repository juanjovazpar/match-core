use crate::shared::types::{
    CommandSeq, EventSeq, OrderId, Price, Quantity, Symbol, Timestamp, TradeId, TradeSeq, UserId,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventEnvelope {
    pub event_seq: EventSeq,
    pub command_seq: CommandSeq,
    pub timestamp: Timestamp,
    pub event: Event,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Event {
    OrderAccepted(OrderAccepted),
    OrderRejected(OrderRejected),
    OrderCancelled(OrderCancelled),
    OrderCancelRejected(OrderCancelRejected),
    TradeExecuted(TradeExecuted),
    OrderPartiallyFilled(OrderPartiallyFilled),
    OrderFilled(OrderFilled),
    OrderExpired(OrderExpired),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderAccepted {
    pub order_id: OrderId,
    pub user_id: UserId,
    pub symbol: Symbol,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderRejected {
    pub order_id: OrderId,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderCancelled {
    pub order_id: OrderId,
    pub user_id: UserId,
    pub symbol: Symbol,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderCancelRejected {
    pub order_id: OrderId,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeExecuted {
    pub trade_seq: TradeSeq,
    pub trade_id: TradeId,
    pub symbol: Symbol,
    pub price: Price,
    pub quantity: Quantity,
    pub maker_order_id: OrderId,
    pub taker_order_id: OrderId,
    pub timestamp: Timestamp,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderPartiallyFilled {
    pub order_id: OrderId,
    pub filled_quantity: Quantity,
    pub remaining_quantity: Quantity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderFilled {
    pub order_id: OrderId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderExpired {
    pub order_id: OrderId,
}
