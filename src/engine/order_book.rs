use std::collections::{BinaryHeap, HashMap, VecDeque};
use std::cmp::{Ordering, Reverse};

use super::order::Order;
use super::trade::Trade;
use crate::engine::order::{Price, Quantity, Side};

pub type OrderQueue = BinaryHeap<TimePriority>;
pub type OrderMap = HashMap<Price, OrderQueue>;
pub type BidQueue = BinaryHeap<Price>;
pub type AskQueue = BinaryHeap<Reverse<Price>>;

// TimePriority will work as a wrapper to allow sorting
// orders by timestamp without alter Order itself.
// Older timestamp will be first.
#[derive(Eq, PartialEq)]
struct TimePriority(Order);
impl Ord for TimePriority {
    fn cmp(&self, other: &Self) -> Ordering {
        other.0.timestamp.cmp(&self.0.timestamp)
    }
}
impl PartialOrd for TimePriority {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub struct OrderBook {
    pub bids: OrderMap,
    pub asks: OrderMap,
    pub bids_queue: BidQueue,
    pub asks_queue: AskQueue,
    pub last_price: Price
}
impl OrderBook {
    pub fn new() -> Self {
        Self {
            bids: OrderMap::new(),
            asks: OrderMap::new(),
            bids_queue: BidQueue::new(),
            asks_queue: AskQueue::new(),
            last_price: 0,
        }
    }

    fn get_best_price(&self, order: &Order) -> Option<Price> {
        match order.side {
            Side::Bid => self.asks_queue.peek().map(|r| r.0),
            Side::Ask => self.bids_queue.peek().copied(),
        }
    }

    fn push(&mut self, order: Order) {
        let map = match order.side {
            Side::Bid => {
                self.bids_queue.push(order.price);
                &mut self.bids
            }
            Side::Ask => {
                self.asks_queue.push(Reverse(order.price));
                &mut self.asks
            }
        };

        map.entry(order.price)
            .or_default()
            .push(TimePriority(order.clone()));
    }

    pub fn execute(&mut self, mut order: Order) {
        while !order.is_complete() {
            let best_price = match self.get_best_price(&order) {
                Some(price) => price,
                None => {
                    self.push(order);
                    break; // TODO: Return message 
                } 
            };

            if best_price > order.price {
                self.push(order);
                break; // TODO: Return message
            }

            let map = match order.side {
                Side::Bid => &mut self.asks,
                Side::Ask => &mut self.bids,
            };
            let candidates = match map.get_mut(&best_price) {
                Some(queue) => queue,
                None => {
                    self.push(order);
                    break;
                }
            };

            while !order.is_complete() && !candidates.is_empty() {
                if let Some(mut item) = candidates.pop() {
                    let candidate = &mut item.0;
                    let quantity = std::cmp::min(
                        order.get_pending(),
                        candidate.get_pending()
                    );      

                    candidate.execute(quantity);
                    order.execute(quantity);

                    if !candidate.is_complete() {
                        candidates.push(TimePriority(candidate.clone()));
                    }

                    self.last_price = best_price;
                    // Emit a Trade::new(order.id, candidate.id, quantity, best_price)
                } else {
                    break;
                }
            }

            if candidates.is_empty() {
                // Remove from prices queue
                map.remove(&best_price);
            }
        }
    }
}