use crate::order::{AccountId, Order, OrderId, Price, Quantity, Side};
use crate::orderbook::OrderBook;

#[derive(Debug, Clone)]
pub struct Trade {
    pub ask_order_id: OrderId,
    pub bid_order_id: OrderId,
    pub ask_account_id: AccountId,
    pub bid_account_id: AccountId,
    pub price: Price,
    pub quantity: Quantity,
}

pub enum OrderUpdate {
    Remove,
    Update(Quantity),
}

pub struct MatchingEngine {
    orderbook: OrderBook,
}

impl MatchingEngine {
    pub fn new() -> Self {
        Self {
            orderbook: OrderBook::new(),
        }
    }

    /// Process a new order, attempting to match it against the orderbook
    /// Returns true if the order was accepted, false if rejected
    pub fn process_order(&mut self, order: Order) -> Vec<Trade> {
        match order.side {
            Side::Bid => self.process_bid(order),
            Side::Ask => self.process_ask(order),
        }
    }

    /// Process a bid order, returns the executed trades.
    fn process_bid(&mut self, mut bid: Order) -> Vec<Trade> {
        // First, collect all the matches and updates we need to make
        let (trades, order_updates) = self.find_bid_matches(&mut bid);

        // Then apply all updates atomically
        if !trades.is_empty() {
            for (order_id, update) in order_updates {
                // If there's a partial fill, we need to update the order quantity
                match update {
                    OrderUpdate::Remove => {
                        self.orderbook.remove_order(order_id, Side::Ask, bid.price);
                    }
                    OrderUpdate::Update(new_qty) => {
                        self.orderbook
                            .update_order_quantity(order_id, Side::Ask, new_qty);
                    }
                }
            }
        } else {
            self.orderbook.insert_order(bid.clone());
        }
        trades
    }

    /// Separate matching logic from update logic
    ///
    /// Also updates the bid quantity to the remaining quantity
    fn find_bid_matches(&self, bid: &mut Order) -> (Vec<Trade>, Vec<(OrderId, OrderUpdate)>) {
        let mut trades = Vec::new();
        let mut updates = Vec::new();
        let mut remaining_qty = bid.quantity.get();

        // Find all matches until we run out of quantity or there are no more asks with a price <= bid.price
        while remaining_qty > 0 {
            let best_ask = match self.orderbook.get_asks().next() {
                Some((price, orders)) if price.get() <= bid.price.get() => Some((*price, orders)),
                _ => None,
            };

            match best_ask {
                Some((ask_price, ask_orders)) => {
                    for ask in ask_orders.iter() {
                        let match_qty = std::cmp::min(remaining_qty, ask.quantity.get());
                        if match_qty > 0 {
                            trades.push(Trade {
                                price: ask_price,
                                quantity: Quantity::new(match_qty),
                                ask_order_id: ask.id,
                                bid_order_id: bid.id,
                                ask_account_id: ask.account_id.clone(),
                                bid_account_id: bid.account_id.clone(),
                            });

                            // Record the update needed
                            if ask.quantity.get() == match_qty {
                                updates.push((ask.id, OrderUpdate::Remove));
                            } else {
                                updates.push((
                                    ask.id,
                                    OrderUpdate::Update(Quantity::new(
                                        ask.quantity.get() - match_qty,
                                    )),
                                ));
                            }

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

        // Update the bid quantity to the remaining quantity
        bid.quantity = Quantity::new(remaining_qty);

        (trades, updates)
    }

    /// Find all the matches for an ask order
    ///
    /// Also updates the ask quantity to the remaining quantity
    fn find_ask_matches(&self, ask: &mut Order) -> (Vec<Trade>, Vec<(OrderId, OrderUpdate)>) {
        let mut trades = Vec::new();
        let mut updates = Vec::new();
        let mut remaining_qty = ask.quantity.get();

        // Find all matches until we run out of quantity or there are no more bids with a price >= ask.price
        while remaining_qty > 0 {
            // Get best bid
            let best_bid = match self.orderbook.get_bids().next() {
                Some((price, _)) if price.to_price().get() >= ask.price.get() => Some(price),
                _ => None,
            };

            match best_bid {
                Some(bid_price) => {
                    // Match against orders at this price level
                    let bid_orders = self.orderbook.get_bids().next().map(|(_, orders)| orders);
                    if let Some(orders) = bid_orders {
                        for bid in orders.iter() {
                            let match_qty = std::cmp::min(remaining_qty, bid.quantity.get());
                            if match_qty > 0 {
                                trades.push(Trade {
                                    price: bid_price.to_price(),
                                    quantity: Quantity::new(match_qty),
                                    ask_order_id: ask.id,
                                    bid_order_id: bid.id,
                                    ask_account_id: ask.account_id.clone(),
                                    bid_account_id: bid.account_id.clone(),
                                });

                                // Record the update needed
                                if bid.quantity.get() == match_qty {
                                    updates.push((bid.id, OrderUpdate::Remove));
                                } else {
                                    updates.push((
                                        bid.id,
                                        OrderUpdate::Update(Quantity::new(
                                            bid.quantity.get() - match_qty,
                                        )),
                                    ));
                                }
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

        // Update the ask quantity to the remaining quantity
        ask.quantity = Quantity::new(remaining_qty);

        (trades, updates)
    }

    /// Process an ask order
    ///
    /// Returns the trades.
    fn process_ask(&mut self, mut ask: Order) -> Vec<Trade> {
        let (trades, updates) = self.find_ask_matches(&mut ask);

        // Handle the results
        if !trades.is_empty() {
            for (order_id, update) in updates {
                match update {
                    OrderUpdate::Remove => {
                        self.orderbook.remove_order(order_id, Side::Bid, ask.price);
                    }
                    OrderUpdate::Update(new_qty) => {
                        self.orderbook
                            .update_order_quantity(order_id, Side::Bid, new_qty);
                    }
                }
            }
        } else {
            self.orderbook.insert_order(ask.clone());
        }
        trades
    }

    /// Cancel an order by its ID. Returns the order if it was found and removed.
    pub fn cancel_order(&mut self, order_id: OrderId, side: Side, price: Price) -> Option<Order> {
        self.orderbook.remove_order(order_id, side, price)
    }
}

#[cfg(test)]
mod tests {
    use crate::order::{AccountId, OrderId, Timestamp};

    use super::*;

    #[test]
    fn test_matching() {
        let mut engine = MatchingEngine::new();

        // Add some asks
        engine.process_order(Order::new(
            OrderId::new(1),
            Price::new(100),
            Quantity::new(10),
            Side::Ask,
            AccountId::new("trader1".to_string()),
            Timestamp::new(1),
        ));
        engine.process_order(Order::new(
            OrderId::new(2),
            Price::new(101),
            Quantity::new(5),
            Side::Ask,
            AccountId::new("trader2".to_string()),
            Timestamp::new(2),
        ));

        // Add a matching bid
        engine.process_order(Order::new(
            OrderId::new(3),
            Price::new(101),
            Quantity::new(7),
            Side::Bid,
            AccountId::new("trader3".to_string()),
            Timestamp::new(3),
        ));

        // Should get a partial fill
        engine.process_order(Order::new(
            OrderId::new(4),
            Price::new(101),
            Quantity::new(2),
            Side::Bid,
            AccountId::new("trader4".to_string()),
            Timestamp::new(4),
        ));
    }
}
