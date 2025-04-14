use std::collections::BTreeMap;

use crate::order::{NegatedPrice, Order, OrderId, Price, Quantity, Side};

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
                let negated_price: NegatedPrice = NegatedPrice::new(bid_order.price.get());

                // Update the order price to the negated price
                bid_order.price = negated_price.to_price();

                // Insert the order into the orderbook
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
    pub fn remove_order(&mut self, order_id: OrderId, side: Side, price: Price) -> Option<Order> {
        let orders = match side {
            Side::Bid => self.bids.get_mut(&NegatedPrice::from_price(price))?,
            Side::Ask => self.asks.get_mut(&price)?,
        };

        if let Some(pos) = orders.iter().position(|o| o.id == order_id) {
            // Remove the order from the orderbook
            let order = orders.remove(pos);

            // If there are no more orders at this price, remove the price from the orderbook
            if orders.is_empty() {
                match order.side {
                    Side::Bid => self.bids.remove(&NegatedPrice::from_price(order.price)),
                    Side::Ask => self.asks.remove(&order.price),
                };
            }
            Some(order)
        } else {
            None
        }
    }

    /// Updates the quantity of an order in the orderbook
    pub fn update_order_quantity(&mut self, order_id: OrderId, side: Side, new_qty: Quantity) {
        // Check bids first
        match side {
            Side::Bid => {
                for orders_at_price in self.bids.values_mut() {
                    if let Some(order) = orders_at_price.iter_mut().find(|o| o.id == order_id) {
                        order.quantity = new_qty;
                        break;
                    }
                }
            }
            Side::Ask => {
                for orders_at_price in self.asks.values_mut() {
                    if let Some(order) = orders_at_price.iter_mut().find(|o| o.id == order_id) {
                        order.quantity = new_qty;
                        break;
                    }
                }
            }
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
    pub fn get_asks(&self) -> impl Iterator<Item = (&Price, &Vec<Order>)> {
        self.asks.iter()
    }

    /// Get the best bid price.
    ///
    /// The bid prices are stored negated (so that the BTreeMap is a min-heap).
    /// Returns the original price.
    pub fn get_best_bid(&self) -> Option<u64> {
        self.bids.first_key_value().map(|(k, _)| k.to_price().get())
    }

    /// Get the best ask price.
    pub fn get_best_ask(&self) -> Option<u64> {
        self.asks.first_key_value().map(|(k, _)| k.get())
    }
}

#[cfg(test)]
mod tests {

    use crate::order::{AccountId, OrderId, Quantity, Timestamp};

    use super::*;

    #[test]
    fn test_order_sorting() {
        let mut ob = OrderBook::new();

        // Add some bids
        ob.insert_order(Order::new(
            OrderId::new(1),
            Price::new(100),
            Quantity::new(10),
            Side::Bid,
            AccountId::new("trader1".to_string()),
            Timestamp::new(1),
        ));

        // Best bid should be 101
        assert_eq!(ob.get_best_bid(), Some(101));

        // Add some asks
        ob.insert_order(Order::new(
            OrderId::new(3),
            Price::new(102),
            Quantity::new(7),
            Side::Ask,
            AccountId::new("trader3".to_string()),
            Timestamp::new(3),
        ));

        // Best ask should be 102
        assert_eq!(ob.get_best_ask(), Some(102));

        // Verify no crossing
        assert!(ob.get_best_bid().unwrap() < ob.get_best_ask().unwrap());

        // Add crossing order
        ob.insert_order(Order::new(
            OrderId::new(5),
            Price::new(102),
            Quantity::new(2),
            Side::Bid,
            AccountId::new("trader5".to_string()),
            Timestamp::new(5),
        ));
        // Best bid should now be 102, crossing with best ask
        assert_eq!(ob.get_best_bid(), Some(102));
        assert_eq!(ob.get_best_ask(), Some(102));

        // Remove an order
        ob.remove_order(OrderId::new(1), Side::Bid, Price::new(100));

        // Verify remaining bid at 101 and 102
        let bids: Vec<u64> = ob.get_bids().map(|(k, _)| k.to_price().get()).collect();
        assert_eq!(bids, vec![102, 101]);
    }
}
