use crate::{
    account_manager::AccountManager,
    asset::Asset,
    market::{Market, Pair},
    order::{AccountId, Order, OrderId, Price, Side},
};
use anyhow::Result;
use std::collections::HashMap;

pub struct Exchange {
    pub markets: HashMap<Pair, Market>,
    pub account_manager: AccountManager,
}

impl Exchange {
    pub fn new() -> Self {
        Exchange {
            markets: HashMap::new(),
            account_manager: AccountManager::new(),
        }
    }

    pub fn add_market(&mut self, market: Market) {
        self.markets.insert(market.pair, market);
    }

    /// Add a balance to an account
    ///
    /// # Arguments
    ///
    /// * `account_id` - The ID of the account to add the balance to
    /// * `asset` - The asset to add the balance to
    /// * `amount` - The amount of the balance to add
    pub fn add_balance(&mut self, account_id: AccountId, asset: Asset, amount: u64) {
        self.account_manager.add_balance(account_id, asset, amount);
    }

    /// Remove a balance from an account
    ///
    /// # Arguments
    ///
    /// * `account_id` - The ID of the account to remove the balance from
    /// * `asset` - The asset to remove the balance from
    /// * `amount` - The amount of the balance to remove
    pub fn remove_balance(
        &mut self,
        account_id: AccountId,
        asset: Asset,
        amount: u64,
    ) -> Result<()> {
        self.account_manager
            .remove_balance(account_id, asset, amount)
    }

    /// Get the balance of an account
    ///
    /// # Arguments
    ///
    /// * `account_id` - The ID of the account to get the balance of
    /// * `asset` - The asset to get the balance of
    pub fn get_balance(&self, account_id: AccountId, asset: Asset) -> Result<u64> {
        self.account_manager.get_balance(account_id, asset)
    }

    /// Post an order
    ///
    /// # Arguments
    ///
    /// * `order` - The order to post
    /// * `pair` - The pair of the order
    pub fn post_order(&mut self, order: Order, pair: Pair) -> Result<()> {
        if order.side == Side::Bid {
            self.remove_balance(
                order.account_id.clone(),
                pair.numeraire,
                order.quantity.get() * order.price.get(),
            )?;
        } else {
            self.remove_balance(order.account_id.clone(), pair.base, order.quantity.get())?;
        }

        let trades = self
            .markets
            .entry(pair)
            .or_insert(Market::new(pair))
            .process_order(order);

        for trade in trades {
            // Ask side receives numeraire
            self.add_balance(
                trade.ask_account_id,
                pair.numeraire,
                trade.quantity.get() * trade.price.get(),
            );

            // Bid side receives base
            self.add_balance(trade.bid_account_id, pair.base, trade.quantity.get());
        }
        Ok(())
    }

    /// Cancel an order
    ///
    /// # Arguments
    ///
    /// * `order_id` - The ID of the order to cancel
    /// * `price` - The price of the order
    /// * `side` - The side of the order
    pub fn cancel_order(
        &mut self,
        order_id: OrderId,
        price: Price,
        side: Side,
        pair: Pair,
    ) -> Result<()> {
        let order = self
            .markets
            .entry(pair)
            .or_insert(Market::new(pair))
            .cancel_order(order_id, side, price);

        if let Some(order) = order {
            if side == Side::Bid {
                self.add_balance(
                    order.account_id,
                    pair.numeraire,
                    order.quantity.get() * order.price.get(),
                );
            } else {
                self.add_balance(order.account_id, pair.base, order.quantity.get());
            }
            Ok(())
        } else {
            Err(anyhow::anyhow!("Order not found"))
        }
    }
}
