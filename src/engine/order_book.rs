use std::collections::{BTreeMap, VecDeque};
use std::cmp::Ordering;
use uuid::Uuid;

use super::order::Order;
use super::trade::Trade;

pub struct OrderBook {
    asks: BTreeMap<u32, VecDeque<Order>>,
    bids: BTreeMap<u32, VecDeque<Order>>,
}
impl OrderBook {
    fn push_order(tree: &mut BTreeMap<u32, VecDeque<Order>>, order: Order) {
        tree.entry(order.price)
            .or_insert(VecDeque::new())
            .push_back(order);
    }

    pub fn new() -> OrderBook {
        OrderBook {
            asks: BTreeMap::new(),
            bids: BTreeMap::new()
        }
    }

    pub fn push_ask(&mut self, order: Order) {
        OrderBook::push_order(&mut self.asks, order);
    }

    pub fn push_bid(&mut self, order: Order) {
        OrderBook::push_order(&mut self.bids, order);
    }

    pub fn bid(&mut self, mut bid: Order) -> VecDeque<Trade> {
        let mut trades: VecDeque<Trade> = VecDeque::new();

        while !bid.is_complete() {
            let lowest_asked_price = match self.asks.keys().next() {
                Some(&price) => price,
                None => {
                    // There is not any ask yet
                    self.push_bid(bid);
                    break;
                }
            };

            // There is not any matching ask yet
            if lowest_asked_price > bid.price {
                self.push_bid(bid);
                break;
            }

            let orders_at_lowest_price = self.asks.get_mut(&lowest_asked_price).unwrap();

            while !bid.is_complete() && !orders_at_lowest_price.is_empty() {
                // Pop order from queue
                let mut ask_order = orders_at_lowest_price.pop_front().unwrap();
                let amount = match bid.get_pending_amount().cmp(&ask_order.get_pending_amount()) {
                    Ordering::Less | Ordering::Equal => bid.get_pending_amount(),
                    Ordering::Greater => ask_order.get_pending_amount(),
                };
                let price = ask_order.price;

                bid.execute_amount(amount);
                ask_order.execute_amount(amount);

                trades.push_back(Trade::new(ask_order.id, bid.id, amount, price));

                if !ask_order.is_complete() {
                    // Push order back to the queue
                    orders_at_lowest_price.push_front(ask_order);
                }
            }
            
            // If orders at this price has been consumed totally,
            // remove the price vector from the asks tree
            if orders_at_lowest_price.len() == 0 {
                self.asks.remove(&lowest_asked_price);
            }
        }

        trades
    }
}