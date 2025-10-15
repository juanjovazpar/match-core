use std::collections::{BTreeSet, HashMap};
use std::cmp::{Reverse};

use crate::engine::order::{Price, Side};
use super::linked_hashmap::LinkedHashmap;
use super::order::Order;
use super::trade::Trade;

pub type OrderQueue = LinkedHashmap<Order>;
pub type OrderMap = HashMap<Price, OrderQueue>;
pub type BidQueue = BTreeSet<Reverse<Price>>; // Ordered greatest to smallest
pub type AskQueue = BTreeSet<Price>; // Ordered smallest to greatest

/* 
    `OrderBook` maintains the current state of a trading book with bids and asks.

    - `bids` and `asks` store orders grouped by price.
    - `bids_queue` and `asks_queue` track available price levels in sorted order
       (highest bid, lowest ask).
*/
pub struct OrderBook {
    bids: OrderMap,
    asks: OrderMap,
    bids_queue: BidQueue,
    asks_queue: AskQueue,
}
impl OrderBook {
    pub fn new() -> Self {
        Self {
            bids: OrderMap::new(),
            asks: OrderMap::new(),
            bids_queue: BidQueue::new(),
            asks_queue: AskQueue::new(),
        }
    }

    fn get_best_price(&self, order: &Order) -> Option<Price> {
        match order.side {
            Side::Bid => self.asks_queue.first().copied(),
            Side::Ask => self.bids_queue.first().map(|r| r.0),
        }
    }

    fn push(&mut self, order: Order) {
        match order.side {
            Side::Bid => {
                self.bids_queue.insert(Reverse(order.price));
                self.bids.entry(order.price).or_default().push(order);
            }
            Side::Ask => {
                self.asks_queue.insert(order.price);
                self.asks.entry(order.price).or_default().push(order);
            }
        }
    }

    pub fn execute(&mut self, mut order: Order) -> Vec<Trade> {
        let mut trades: Vec<Trade> = Vec::new();

        while !order.is_complete() {
            let best_price = match self.get_best_price(&order) {
                Some(price) => price,
                None => {
                    self.push(order);
                    break;
                }
            };

            match order.side {
                Side::Bid if best_price > order.price => {
                    self.push(order);
                    break;
                }
                Side::Ask if best_price < order.price => {
                    self.push(order);
                    break;
                }
                _ => {}
            }

            let map = match order.side {
                Side::Bid => &mut self.asks,
                Side::Ask => &mut self.bids,
            };

            let queue = match map.get_mut(&best_price) {
                Some(q) => q,
                None => {
                    self.push(order);
                    break;
                }
            };

            while !order.is_complete() && !queue.is_empty() {
                if let Some(mut candidate) = queue.pop() {
                    let candidate_id = candidate.id;
                    let quantity = std::cmp::min(order.get_pending(), candidate.get_pending());

                    candidate.execute(quantity);
                    order.execute(quantity);

                    if !candidate.is_complete() {
                        // push back if partially filled
                        queue.push_first(candidate);
                    }

                    let trade = Trade::new(order.id, candidate_id, quantity, best_price);
                    trades.push(trade);
                } else {
                    break;
                }
            }

            if queue.is_empty() {
                map.remove(&best_price);
                match order.side {
                    Side::Bid => self.asks_queue.remove(&best_price),
                    Side::Ask => self.bids_queue.remove(&Reverse(best_price)),
                };
            }
        }
        
        trades
    }

    pub fn cancel(&mut self, order: Order) {
        match order.side {
            Side::Bid => {
                if let Some(queue) = self.bids.get_mut(&order.price) {
                    queue.remove(&order.id);

                    if queue.is_empty() {
                        self.bids.remove(&order.price);
                        self.bids_queue.remove(&Reverse(order.price));
                    }
                } else {
                    self.bids_queue.remove(&Reverse(order.price));
                }
            }
            Side::Ask => {
                if let Some(queue) = self.asks.get_mut(&order.price) {
                    queue.remove(&order.id);

                    if queue.is_empty() {
                        self.asks.remove(&order.price);
                        self.asks_queue.remove(&order.price);
                    }
                } else {
                    self.asks_queue.remove(&order.price);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::order::{Side, Mode};
    use uuid::Uuid;

    #[test]
    fn creation() {
        let book = OrderBook::new();
        let order = Order::new(Uuid::new_v4(), 100, 10, Side::Ask, Mode::Limit);

        assert!(book.bids.is_empty(), "bids map should be empty");
        assert!(book.asks.is_empty(), "asks map should be empty");
        assert!(book.bids_queue.is_empty(), "bids queue should be empty");
        assert!(book.asks_queue.is_empty(), "asks queue should be empty");
        assert_eq!(book.get_best_price(&order), None);
    }

    #[test]
    fn execute_without_trades() {
        let mut book = OrderBook::new();
        let best_price = 10;
        let ask_order_1 = Order::new(Uuid::new_v4(), 100, best_price, Side::Ask, Mode::Market);
        let ask_order_2 = Order::new(Uuid::new_v4(), 100, 15, Side::Ask, Mode::Market);
        let bid_order_1 = Order::new(Uuid::new_v4(), 100, 9, Side::Bid, Mode::Market);

        book.execute(ask_order_1);
        book.execute(ask_order_2);
        
        assert_eq!(book.get_best_price(&bid_order_1), Some(best_price));

        let trades = book.execute(bid_order_1);

        assert_eq!(trades.len(), 0);
    }

    #[test]
    fn execute_with_one_trade() {
        let mut book = OrderBook::new();
        let best_price = 10;
        let ask_order_1 = Order::new(Uuid::new_v4(), 100, best_price, Side::Ask, Mode::Market);
        let ask_order_2 = Order::new(Uuid::new_v4(), 100, 15, Side::Ask, Mode::Market);
        let bid_order_1 = Order::new(Uuid::new_v4(), 100, best_price, Side::Bid, Mode::Market);

        book.execute(ask_order_1);
        book.execute(ask_order_2);
        
        assert_eq!(book.get_best_price(&bid_order_1), Some(best_price));

        let trades = book.execute(bid_order_1);

        assert_eq!(trades.len(), 1);
    }

    #[test]
    fn execute_with_two_trades() {
        let mut book = OrderBook::new();
        let ask_order_1 = Order::new(Uuid::new_v4(), 100, 10, Side::Ask, Mode::Market);
        let ask_order_2 = Order::new(Uuid::new_v4(), 100, 9, Side::Ask, Mode::Market);
        let bid_order_1 = Order::new(Uuid::new_v4(), 200, 12, Side::Bid, Mode::Market);

        book.execute(ask_order_1);
        book.execute(ask_order_2);
        
        let trades = book.execute(bid_order_1);

        assert_eq!(trades.len(), 2);
    }

    #[test]
    fn cancel_candidate() {
        let mut book = OrderBook::new();
        let ask_order_1 = Order::new(Uuid::new_v4(), 100, 10, Side::Ask, Mode::Market);
        let ask_order_2 = Order::new(Uuid::new_v4(), 100, 9, Side::Ask, Mode::Market);
        let bid_order_1 = Order::new(Uuid::new_v4(), 200, 12, Side::Bid, Mode::Market);
        let ask_order_clone = ask_order_2.clone();

        book.execute(ask_order_1);
        book.execute(ask_order_2);

        book.cancel(ask_order_clone);
        
        let trades = book.execute(bid_order_1);

        assert_eq!(trades.len(), 1);
    }
}