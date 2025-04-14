use std::collections::HashMap;

use crate::{
    account_manager::AccountManager,
    asset::Asset,
    matching::{MatchingEngine, Trade},
    order::{AccountId, Order, OrderId, Price, Side},
};
use anyhow::Result;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Pair {
    pub numeraire: Asset,
    pub base: Asset,
}
pub struct Market {
    pub pair: Pair,
    pub matching_engine: MatchingEngine,
}

impl Market {
    pub fn new(pair: Pair) -> Self {
        Market {
            pair,
            matching_engine: MatchingEngine::new(),
        }
    }

    /// Processes an order, returning the trades.
    pub fn process_order(&mut self, order: Order) -> Vec<Trade> {
        self.matching_engine.process_order(order)
    }

    pub fn cancel_order(&mut self, order_id: OrderId, side: Side, price: Price) -> Option<Order> {
        self.matching_engine.cancel_order(order_id, side, price)
    }
}
