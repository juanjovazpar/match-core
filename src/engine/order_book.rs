use std::collections::{BTreeSet, HashMap};
use std::cmp::{Reverse};

use super::linked_hashmap::LinkedHashmap;
use super::order::Order;
use crate::engine::order::{Price, Side};
use super::trade::Trade;

pub type OrderQueue = LinkedHashmap<Order>;
pub type OrderMap = HashMap<Price, OrderQueue>;
pub type BidQueue = BTreeSet<Price>;
pub type AskQueue = BTreeSet<Reverse<Price>>;

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
            Side::Bid => self.asks_queue.iter().next().map(|r| r.0),
            Side::Ask => self.bids_queue.iter().next_back().copied(),
        }
    }

    fn push(&mut self, order: Order) {
        let map = match order.side {
            Side::Bid => {
                self.bids_queue.insert(order.price);
                &mut self.bids
            }
            Side::Ask => {
                self.asks_queue.insert(Reverse(order.price));
                &mut self.asks
            }
        };

        map.entry(order.price)
            .or_default()
            .push(order);
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
                        queue.push_first(candidate); // push back if partially filled
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
                    Side::Bid => self.asks_queue.remove(&Reverse(best_price)),
                    Side::Ask => self.bids_queue.remove(&best_price),
                };
            }
        }
        
        trades
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::order::{Side, Mode};
    use uuid::Uuid;

    #[test]
    fn creation() {
        let mut book = OrderBook::new();
        let order = Order::new(Uuid::new_v4(), 100, 10, Side::Ask, Mode::Limit);

        let trades = book.execute(order);

        assert_eq!(trades.len(), 0);
    }
}