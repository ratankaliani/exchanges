use crate::{
    account::Account,
    asset::Asset,
    order::{AccountId, Quantity},
};
use anyhow::Result;
use std::{
    collections::HashMap,
    ops::{Add, Sub},
};

pub struct AccountManager {
    accounts: HashMap<AccountId, Account>,
    // todo: add overall positions and risk limits later.
}

/// Print all balances for all accounts
impl AccountManager {
    pub fn print_balances(&self) {
        // Get all unique assets across accounts
        let mut assets: Vec<Asset> = self
            .accounts
            .values()
            .flat_map(|acc| acc.balances.keys())
            .cloned()
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        assets.sort_by_key(|a| a.symbol);

        // Calculate column widths
        let account_width = self
            .accounts
            .keys()
            .map(|id| format!("{:?}", id).len())
            .max()
            .unwrap_or(7)
            .max(7);
        let asset_widths: Vec<_> = assets.iter().map(|a| a.symbol.len().max(6)).collect();

        // Print header
        print!("{:width$}", "Account", width = account_width);
        for (asset, width) in assets.iter().zip(&asset_widths) {
            print!(" | {:^width$}", asset.symbol, width = width);
        }
        println!();

        // Print separator
        print!("{}", "-".repeat(account_width));
        for width in &asset_widths {
            print!("-|-{}", "-".repeat(*width));
        }
        println!();

        // Print balances
        for (account_id, account) in &self.accounts {
            print!("{:width$?}", account_id, width = account_width);
            for (asset, width) in assets.iter().zip(&asset_widths) {
                let balance = account.balances.get(asset).map_or(0, |q| q.get());
                print!(" | {:>width$}", balance, width = width);
            }
            println!();
        }
    }
}

impl AccountManager {
    pub fn new() -> Self {
        Self {
            accounts: HashMap::new(),
        }
    }

    /// Add a balance to an account
    ///
    /// # Arguments
    ///
    /// * `account_id` - The ID of the account to add the balance to
    /// * `asset` - The asset to add the balance to
    /// * `amount` - The amount of the balance to add
    pub fn add_balance(&mut self, account_id: AccountId, asset: Asset, amount: u64) {
        let account = self
            .accounts
            .entry(account_id.clone())
            .or_insert(Account::new(account_id));
        let balance = account.balances.entry(asset).or_insert(Quantity::new(0));
        *balance = balance.add(Quantity::new(amount));
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
        let account = self
            .accounts
            .get_mut(&account_id)
            .ok_or(anyhow::anyhow!("Account not found"))?;
        let balance = account.balances.entry(asset).or_insert(Quantity::new(0));
        if balance.get() < amount {
            return Err(anyhow::anyhow!("Insufficient balance"));
        }
        let new_balance = balance.sub(Quantity::new(amount));
        account.balances.insert(asset, new_balance);
        Ok(())
    }

    /// Get the balance of an account
    ///
    /// # Arguments
    ///
    /// * `account_id` - The ID of the account to get the balance of
    /// * `asset` - The asset to get the balance of
    pub fn get_balance(&self, account_id: AccountId, asset: Asset) -> Result<u64> {
        let account = self
            .accounts
            .get(&account_id)
            .ok_or(anyhow::anyhow!("Account not found"))?;
        Ok(account.balances.get(&asset).unwrap().get())
    }
}
