use serde::{Deserialize, Serialize};

pub type CommandId = String;
pub type CommandSeq = u64;
pub type EventSeq = u64;
pub type Timestamp = u64;
pub type TradeSeq = u64;
pub type TradeId = String;
pub type OrderId = String;
pub type Price = f64;
pub type Quantity = f64;
pub type UserId = String;
pub type Symbol = String;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Side {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Type {
    Limit,
    Market,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TimeInForce {
    GTC,
    IOC,
    FOK,
}
