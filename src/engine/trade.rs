use chrono::Utc;
use uuid::Uuid;

pub struct Trade {
    pub id: Uuid,
    pub ask_id: Uuid,
    pub bid_id: Uuid,
    pub amount: u32,
    pub price: u32,
    pub timestamp: i64,
}
impl Trade {
    pub fn new(ask_id: Uuid, bid_id: Uuid, amount: u32, price: u32) -> Trade {
        Trade {
            id: Uuid::new_v4(),
            ask_id,
            bid_id,
            amount,
            price,
            timestamp: Utc::now().timestamp_millis()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trade_creation() {
        let ask_id = Uuid::new_v4();
        let bid_id = Uuid::new_v4();
        let amount = 100;
        let price = 50;

        let trade = Trade::new(ask_id, bid_id, amount, price);

        assert_eq!(trade.ask_id, ask_id);
        assert_eq!(trade.bid_id, bid_id);
        assert_eq!(trade.amount, amount);
        assert_eq!(trade.price, price);
        assert_ne!(trade.timestamp, 1);

        assert!(Uuid::parse_str(&trade.id.to_string()).is_ok());
    }
}