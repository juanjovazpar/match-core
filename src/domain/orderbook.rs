use std::collections::{BTreeMap, VecDeque};

use ordered_float::OrderedFloat;

use crate::shared::command::Side;

use super::order::Order;

pub struct OrderBook {
    pub bids: BTreeMap<OrderedFloat<f64>, VecDeque<Order>>,
    pub asks: BTreeMap<OrderedFloat<f64>, VecDeque<Order>>,
}
impl OrderBook {
    pub fn new() -> Self {
        Self {
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
        }
    }

    pub fn add_order(&mut self, order: Order) {
        let book = match order.side {
            Side::Buy => &mut self.bids,
            Side::Sell => &mut self.asks,
        };

        book.entry(OrderedFloat(order.price))
            .or_insert_with(VecDeque::new)
            .push_back(order);
    }
}