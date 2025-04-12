use std::collections::VecDeque;

use crate::order::{Order, Price, Quantity, Side};
use crate::orderbook::OrderBook;

#[derive(Debug, Clone)]
pub struct Trade {
    pub price: Price,
    pub quantity: Quantity,
    pub bid_order_id: u64,
    pub ask_order_id: u64,
}

#[derive(Debug)]
pub enum OrderEvent {
    Accepted(Order),
    Rejected(Order, String),
    Filled(Order, Vec<Trade>),
    PartiallyFilled(Order, Vec<Trade>),
    Cancelled(Order),
}

pub struct MatchingEngine {
    orderbook: OrderBook,
    events: VecDeque<OrderEvent>,
}

impl MatchingEngine {
    pub fn new() -> Self {
        Self {
            orderbook: OrderBook::new(),
            events: VecDeque::new(),
        }
    }

    /// Process a new order, attempting to match it against the orderbook
    /// Returns true if the order was accepted, false if rejected
    pub fn process_order(&mut self, order: Order) -> bool {
        match order.side {
            Side::Bid => self.process_bid(order),
            Side::Ask => self.process_ask(order),
        }
    }

    /// Process a bid order
    fn process_bid(&mut self, mut bid: Order) -> bool {
        // Try to match against asks
        let mut remaining_qty = bid.quantity.0;
        let mut trades = Vec::new();

        while remaining_qty > 0 {
            // Get best ask
            let best_ask = match self.orderbook.get_asks().first_key_value() {
                Some((price, orders)) if price.0 <= bid.price.0 => Some((*price, orders)),
                _ => None,
            };

            match best_ask {
                Some((ask_price, ask_orders)) => {
                    // Match against orders at this price level
                    for ask in ask_orders.iter() {
                        let match_qty = std::cmp::min(remaining_qty, ask.quantity.0);
                        if match_qty > 0 {
                            trades.push(Trade {
                                price: ask_price,
                                quantity: Quantity(match_qty),
                                bid_order_id: bid.id.0,
                                ask_order_id: ask.id.0,
                            });
                            remaining_qty -= match_qty;
                        }
                        if remaining_qty == 0 {
                            break;
                        }
                    }
                }
                None => break,
            }
        }

        // Handle the results
        if !trades.is_empty() {
            if remaining_qty == 0 {
                self.events.push_back(OrderEvent::Filled(bid, trades));
            } else {
                bid.quantity = Quantity(remaining_qty);
                self.orderbook.insert_order(bid.clone());
                self.events
                    .push_back(OrderEvent::PartiallyFilled(bid, trades));
            }
            true
        } else {
            self.orderbook.insert_order(bid.clone());
            self.events.push_back(OrderEvent::Accepted(bid));
            true
        }
    }

    /// Process an ask order
    fn process_ask(&mut self, mut ask: Order) -> bool {
        // Try to match against bids
        let mut remaining_qty = ask.quantity.0;
        let mut trades = Vec::new();

        while remaining_qty > 0 {
            // Get best bid
            let best_bid = match self.orderbook.get_best_bid() {
                Some(price) if price >= ask.price.0 => Some(price),
                _ => None,
            };

            match best_bid {
                Some(bid_price) => {
                    // Match against orders at this price level
                    let bid_orders = self.orderbook.get_bids().next().map(|(_, orders)| orders);
                    if let Some(orders) = bid_orders {
                        for bid in orders.iter() {
                            let match_qty = std::cmp::min(remaining_qty, bid.quantity.0);
                            if match_qty > 0 {
                                trades.push(Trade {
                                    price: Price(bid_price),
                                    quantity: Quantity(match_qty),
                                    bid_order_id: bid.id.0,
                                    ask_order_id: ask.id.0,
                                });
                                remaining_qty -= match_qty;
                            }
                            if remaining_qty == 0 {
                                break;
                            }
                        }
                    }
                }
                None => break,
            }
        }

        // Handle the results
        if !trades.is_empty() {
            if remaining_qty == 0 {
                self.events.push_back(OrderEvent::Filled(ask, trades));
            } else {
                ask.quantity = Quantity(remaining_qty);
                self.orderbook.insert_order(ask.clone());
                self.events
                    .push_back(OrderEvent::PartiallyFilled(ask, trades));
            }
            true
        } else {
            self.orderbook.insert_order(ask.clone());
            self.events.push_back(OrderEvent::Accepted(ask));
            true
        }
    }

    /// Cancel an order by its ID
    pub fn cancel_order(&mut self, order_id: u64, side: Side, price: u64) -> bool {
        if let Some(order) = self.orderbook.remove_order(order_id, side, price) {
            self.events.push_back(OrderEvent::Cancelled(order));
            true
        } else {
            false
        }
    }

    /// Get the next event from the queue, if any
    pub fn next_event(&mut self) -> Option<OrderEvent> {
        self.events.pop_front()
    }
}

#[cfg(test)]
mod tests {
    use crate::order::{OrderId, Timestamp};

    use super::*;

    #[test]
    fn test_matching() {
        let mut engine = MatchingEngine::new();

        // Add some asks
        engine.process_order(Order::new(
            OrderId(1),
            Price(100),
            Quantity(10),
            Side::Ask,
            "trader1".to_string(),
            Timestamp(1),
        ));
        engine.process_order(Order::new(
            OrderId(2),
            Price(101),
            Quantity(5),
            Side::Ask,
            "trader2".to_string(),
            Timestamp(2),
        ));

        // Add a matching bid
        engine.process_order(Order::new(
            OrderId(3),
            Price(101),
            Quantity(7),
            Side::Bid,
            "trader3".to_string(),
            Timestamp(3),
        ));

        // Should get a partial fill
        match engine.next_event() {
            Some(OrderEvent::PartiallyFilled(order, trades)) => {
                assert_eq!(order.id.0, 3);
                assert_eq!(trades.len(), 2);
                assert_eq!(trades[0].quantity.0, 5);
                assert_eq!(trades[1].quantity.0, 2);
            }
            _ => panic!("Expected partial fill"),
        }
    }
}
