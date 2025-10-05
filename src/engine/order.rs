use std::cmp::Ordering;
use chrono::Utc;
use uuid::Uuid;

pub struct Order {
    pub id: Uuid,
    pub owner: String,
    pub amount: u32,
    pub price: u32,
    pub executed_amount: u32,
    pub timestamp: i64,

}
impl Order {
    pub fn new(owner: String, amount: u32, price: u32) -> Order {
        Order {
            id: Uuid::new_v4(),
            owner,
            price,
            amount,
            executed_amount: 0,
            timestamp: Utc::now().timestamp_millis()
        }
    }

    pub fn is_complete(&self) -> bool {
        self.amount <= self.executed_amount
    }

    pub fn get_pending_amount(&self) -> u32 {
        self.amount - self.executed_amount
    }

    // Execute an specific amount from the Order's available amount
    // and return the remainder of the amount that couldn't be executed
    pub fn execute_amount(&mut self, amount: u32) -> u32 {
        let available_amount: u32 = self.get_pending_amount();

        match available_amount.cmp(&amount) {
            Ordering::Less => { 
                self.executed_amount = self.amount;
                amount - available_amount
             },
            Ordering::Equal => {
                self.executed_amount = self.amount;
                0
            },
            Ordering::Greater => {
                self.executed_amount += amount;
                0
            },
        }
    }
}
