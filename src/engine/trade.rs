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