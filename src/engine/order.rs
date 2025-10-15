use std::cmp::Ordering;
use chrono::Utc;
use uuid::Uuid;
use super::linked_hashmap::HasId;

pub type Price = u32;
pub type OrderId = u64;
pub type Quantity = u64;
pub type Timestamp = i64;

#[derive(Eq, Clone, PartialEq, Debug)]
pub enum Side {
    Ask,
    Bid
}

#[derive(Eq, Clone, PartialEq, Debug)]
pub enum Mode {
    Limit,
    Market
}

#[derive(Eq, Clone, PartialEq, Debug)]
pub struct Order {
    pub id: Uuid,
    pub owner: Uuid,
    pub quantity: Quantity,
    pub price: Price,
    pub executed: Quantity,
    pub timestamp: Timestamp,
    pub side: Side,
    pub mode: Mode,
}
impl Order {
    pub fn new(owner: Uuid, quantity: Quantity, price: Price, side: Side, mode: Mode) -> Self {
        Self {
            owner,
            price,
            quantity,
            side,
            mode,
            executed: 0,
            id: Uuid::new_v4(),
            timestamp: Utc::now().timestamp_millis(),
        }
    }

    pub fn is_complete(&self) -> bool {
        self.quantity <= self.executed
    }

    pub fn get_pending(&self) -> Quantity {
        self.quantity - self.executed
    }

    // Execute an specific amount from the Order's available amount
    // and return the remainder of the amount that couldn't be executed
    pub fn execute(&mut self, quantity: Quantity) -> Quantity {
        let pending: Quantity = self.get_pending();

        match pending.cmp(&quantity) {
            Ordering::Less => { 
                self.executed = self.quantity;
                quantity - pending
             },
            Ordering::Equal => {
                self.executed = self.quantity;
                0
            },
            Ordering::Greater => {
                self.executed += quantity;
                0
            },
        }
    }
}
impl HasId for Order {
    type Id = Uuid;

    fn id(&self) -> Self::Id {
        self.id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_order_creation() {
        let owner = Uuid::new_v4();
        let amount: u64 = 100;
        let price = 50;
        let side = Side::Ask;
        let execution = Mode::Limit;

        let order = Order::new(owner.clone(), amount, price, side.clone(), execution.clone());

        assert_eq!(order.owner, owner);
        assert_eq!(order.quantity, amount);
        assert_eq!(order.price, price);
        assert_eq!(order.executed, 0);
        assert_eq!(order.side, side);
        assert_eq!(order.mode, execution);

        assert!(Uuid::parse_str(&order.id.to_string()).is_ok());

        let now = Utc::now().timestamp_millis();
        assert!((now - order.timestamp).abs() < 1000);

        assert_eq!(order.is_complete(), false);
        assert_eq!(order.get_pending(), amount);
    }

    #[test]
    fn test_order_execution() {
        let owner = Uuid::new_v4();
        let amount = 100;
        let price = 50;
        let side = Side::Ask;
        let execution = Mode::Limit;

        let mut order = Order::new(owner.clone(), amount, price, side.clone(), execution.clone());

        assert_eq!(order.is_complete(), false);
        assert_eq!(order.get_pending(), amount);

        let sustract = 10;
        let remainder = order.execute(amount - sustract);

        assert_eq!(order.is_complete(), false);
        assert_eq!(order.get_pending(), sustract);
        assert_eq!(remainder, 0);
    }

    #[test]
    fn test_order_execution_with_remainder() {
        let owner = Uuid::new_v4();
        let amount = 100;
        let price = 50;
        let side = Side::Ask;
        let execution = Mode::Limit;

        let mut order = Order::new(owner.clone(), amount, price, side.clone(), execution.clone());

        assert_eq!(order.is_complete(), false);
        assert_eq!(order.get_pending(), amount);

        let sustract = 110;
        let remainder = order.execute(sustract);

        assert_eq!(order.is_complete(), true);
        assert_eq!(order.get_pending(), 0);
        assert_eq!(remainder, 10);
    }
}