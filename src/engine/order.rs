use std::cmp::Ordering;
use chrono::Utc;
use uuid::Uuid;

#[derive(Clone, PartialEq, Debug)]
pub enum OrderType {
    Limit,
    Market
}

#[derive(Clone, PartialEq, Debug)]
pub struct Order {
    pub id: Uuid,
    pub owner: String,
    pub amount: u32,
    pub price: u32,
    pub executed_amount: u32,
    pub timestamp: i64,
    pub order_type: OrderType,
}
impl Order {
    pub fn new(owner: String, amount: u32, price: u32, order_type: OrderType) -> Order {
        Order {
            id: Uuid::new_v4(),
            owner,
            price,
            amount,
            executed_amount: 0,
            timestamp: Utc::now().timestamp_millis(),
            order_type,
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
        let pending_amount: u32 = self.get_pending_amount();

        match pending_amount.cmp(&amount) {
            Ordering::Less => { 
                self.executed_amount = self.amount;
                amount - pending_amount
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_order_creation() {
        let owner = "Alice".to_string();
        let amount = 100;
        let price = 50;
        let order_type = OrderType::Limit; 

        let order = Order::new(owner.clone(), amount, price, order_type.clone());

        assert_eq!(order.owner, owner);
        assert_eq!(order.amount, amount);
        assert_eq!(order.price, price);
        assert_eq!(order.executed_amount, 0);
        assert_eq!(order.order_type, order_type);

        assert!(Uuid::parse_str(&order.id.to_string()).is_ok());

        let now = Utc::now().timestamp_millis();
        assert!((now - order.timestamp).abs() < 1000);

        assert_eq!(order.is_complete(), false);
        assert_eq!(order.get_pending_amount(), amount);
    }

    #[test]
    fn test_order_execution() {
        let owner = "Alice".to_string();
        let amount = 100;
        let price = 50;
        let order_type = OrderType::Limit;

        let mut order = Order::new(owner.clone(), amount, price, order_type.clone());

        assert_eq!(order.is_complete(), false);
        assert_eq!(order.get_pending_amount(), amount);

        let sustract = 10;
        let remainder = order.execute_amount(amount - sustract);

        assert_eq!(order.is_complete(), false);
        assert_eq!(order.get_pending_amount(), sustract);
        assert_eq!(remainder, 0);
    }

    #[test]
    fn test_order_execution_with_remainder() {
        let owner = "Alice".to_string();
        let amount = 100;
        let price = 50;
        let order_type = OrderType::Limit;

        let mut order = Order::new(owner.clone(), amount, price, order_type.clone());

        assert_eq!(order.is_complete(), false);
        assert_eq!(order.get_pending_amount(), amount);

        let sustract = 110;
        let remainder = order.execute_amount(sustract);

        assert_eq!(order.is_complete(), true);
        assert_eq!(order.get_pending_amount(), 0);
        assert_eq!(remainder, 10);
    }
}