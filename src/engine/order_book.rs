use std::collections::{BTreeMap, VecDeque};
use std::cmp::Ordering;

use super::order::Order;
use super::trade::Trade;

pub struct OrderBook {
    asks: BTreeMap<u32, VecDeque<Order>>,
    bids: BTreeMap<u32, VecDeque<Order>>,
}
impl OrderBook {
    pub fn new() -> OrderBook {
        OrderBook {
            asks: BTreeMap::new(),
            bids: BTreeMap::new()
        }
    }

    fn push_order(tree: &mut BTreeMap<u32, VecDeque<Order>>, order: Order) {
        tree.entry(order.price)
            .or_insert(VecDeque::new())
            .push_back(order);
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

    pub fn ask(&mut self, mut ask: Order) -> VecDeque<Trade> {
        let mut trades: VecDeque<Trade> = VecDeque::new();

        while !ask.is_complete() {
            let highest_offered_price = match self.bids.keys().last() {
                Some(&price) => price,
                None => {
                    // There is not any bid yet
                    self.push_ask(ask);
                    break;
                }
            };

            // There is not any matching bid yet
            if highest_offered_price > ask.price {
                self.push_bid(ask);
                break;
            }

            let orders_at_highest_price = self.asks.get_mut(&highest_offered_price).unwrap();

            while !ask.is_complete() && !orders_at_highest_price.is_empty() {
                // Pop order from queue
                let mut bid_order = orders_at_highest_price.pop_front().unwrap();
                let amount = match ask.get_pending_amount().cmp(&bid_order.get_pending_amount()) {
                    Ordering::Less | Ordering::Equal => ask.get_pending_amount(),
                    Ordering::Greater => bid_order.get_pending_amount(),
                };
                let price = bid_order.price;

                ask.execute_amount(amount);
                bid_order.execute_amount(amount);

                trades.push_back(Trade::new(ask.id, bid_order.id, amount, price));

                if !bid_order.is_complete() {
                    // Push order back to the queue
                    orders_at_highest_price.push_front(bid_order);
                }
            }
            
            // If orders at this price has been consumed totally,
            // remove the price vector from the asks tree
            if orders_at_highest_price.len() == 0 {
                self.asks.remove(&highest_offered_price);
            }
        }

        trades
    }
}

#[cfg(test)]
mod tests {
    use crate::engine::order::OrderType;
    use super::*;

    #[test]
    fn test_orderbook_creation() {
        let asks = BTreeMap::new();
        let bids = BTreeMap::new();

        let order_book: OrderBook = OrderBook::new();

        assert_eq!(order_book.asks, asks);
        assert_eq!(order_book.bids, bids);
    }

    #[test]
    fn test_push_orders() {
        let price = 100;
        let mut order_book: OrderBook = OrderBook::new();
        let order = Order::new(
            "Alice".to_string(),
            100,
            price,
            OrderType::Limit
        );
        let mut expected_collection = BTreeMap::new();
        let mut queue = VecDeque::new();
        let ask_order = order.clone();
        let bid_order = order.clone();

        queue.push_back(order);
        expected_collection.insert(price, queue);

        order_book.push_ask(ask_order);
        assert_eq!(order_book.asks, expected_collection);

        order_book.push_bid(bid_order);
        assert_eq!(order_book.bids, expected_collection);
    }
}