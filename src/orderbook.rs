use std::collections::BTreeMap;

use crate::order::{NegatedPrice, Order, Price, Side};

/// A double-sided orderbook that maintains sorted bids and asks
///
/// Bids are stored with negated prices to maintain descending order (highest first)
/// Asks are stored with natural prices to maintain ascending order (lowest first)
#[derive(Debug, Default)]
pub struct OrderBook {
    bids: BTreeMap<NegatedPrice, Vec<Order>>, // negated price -> orders (ascending)
    asks: BTreeMap<Price, Vec<Order>>,        // price -> orders (ascending)
}

impl OrderBook {
    /// Creates a new empty orderbook
    pub fn new() -> Self {
        Self {
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
        }
    }

    /// Inserts a new order into the orderbook
    ///
    /// For bids, the price is negated to maintain descending order
    /// For asks, the price is stored as-is to maintain ascending order
    pub fn insert_order(&mut self, order: Order) {
        match order.side {
            Side::Bid => {
                let mut bid_order = order;
                let negated_price: NegatedPrice = bid_order.price.into();
                bid_order.price = Price(negated_price.0);
                self.bids
                    .entry(negated_price)
                    .or_insert_with(Vec::new)
                    .push(bid_order);
            }
            Side::Ask => {
                self.asks
                    .entry(order.price)
                    .or_insert_with(Vec::new)
                    .push(order);
            }
        }
    }

    /// Removes an order from the orderbook by its ID, side, and price
    ///
    /// For bids, the price must be provided in its original form (not negated)
    /// Returns the removed order if found, None otherwise
    pub fn remove_order(&mut self, order_id: u64, side: Side, price: u64) -> Option<Order> {
        let orders = match side {
            Side::Bid => self.bids.get_mut(&NegatedPrice::from(Price(price)))?,
            Side::Ask => self.asks.get_mut(&Price(price))?,
        };

        if let Some(pos) = orders.iter().position(|o| o.id.0 == order_id) {
            let order = orders.remove(pos);
            if orders.is_empty() {
                match side {
                    Side::Bid => self.bids.remove(&NegatedPrice::from(Price(price))),
                    Side::Ask => self.asks.remove(&Price(price)),
                };
            }
            Some(order)
        } else {
            None
        }
    }

    /// Get all bids.
    ///
    /// The prices in the bids are negated.
    pub fn get_bids(&self) -> impl Iterator<Item = (&NegatedPrice, &Vec<Order>)> {
        self.bids.iter()
    }

    /// Get all asks.
    ///
    /// The prices are in their original form (not negated).
    pub fn get_asks(&self) -> &BTreeMap<Price, Vec<Order>> {
        &self.asks
    }

    /// Get the best bid price.
    ///
    /// The bid prices are stored negated (so that the BTreeMap is a min-heap).
    /// Returns the original price.
    pub fn get_best_bid(&self) -> Option<u64> {
        self.bids.first_key_value().map(|(k, _)| Price::from(*k).0)
    }

    /// Get the best ask price.
    pub fn get_best_ask(&self) -> Option<u64> {
        self.asks.first_key_value().map(|(k, _)| k.0)
    }
}

#[cfg(test)]
mod tests {

    use crate::order::{OrderId, Quantity, Timestamp};

    use super::*;

    #[test]
    fn test_order_sorting() {
        let mut ob = OrderBook::new();

        // Add some bids
        ob.insert_order(Order::new(
            OrderId(1),
            Price(100),
            Quantity(10),
            Side::Bid,
            "trader1".to_string(),
            Timestamp(1),
        ));

        // Best bid should be 101
        assert_eq!(ob.get_best_bid(), Some(101));

        // Add some asks
        ob.insert_order(Order::new(
            OrderId(3),
            Price(102),
            Quantity(7),
            Side::Ask,
            "trader3".to_string(),
            Timestamp(3),
        ));

        // Best ask should be 102
        assert_eq!(ob.get_best_ask(), Some(102));

        // Verify no crossing
        assert!(ob.get_best_bid().unwrap() < ob.get_best_ask().unwrap());

        // Add crossing order
        ob.insert_order(Order::new(
            OrderId(5),
            Price(102),
            Quantity(2),
            Side::Bid,
            "trader5".to_string(),
            Timestamp(5),
        ));
        // Best bid should now be 102, crossing with best ask
        assert_eq!(ob.get_best_bid(), Some(102));
        assert_eq!(ob.get_best_ask(), Some(102));

        // Remove an order
        ob.remove_order(1, Side::Bid, 100);

        // Verify remaining bid at 101 and 102
        let bids: Vec<u64> = ob.get_bids().map(|(k, _)| Price::from(*k).0).collect();
        assert_eq!(bids, vec![102, 101]);
    }
}
