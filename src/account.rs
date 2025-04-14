use std::collections::HashMap;

use crate::{
    asset::Asset,
    order::{AccountId, OrderId, Price, Quantity, Side},
};

pub struct Account {
    pub id: AccountId,
    pub balances: HashMap<Asset, Quantity>,
    // Orders are stored in a map of asset to a map of order id to (side, price). The quantity can change and is only fulfilled when the order is cancelled.
    pub orders: HashMap<Asset, HashMap<OrderId, (Side, Price)>>,
}

impl Account {
    pub fn new(id: AccountId) -> Self {
        Self {
            id,
            balances: HashMap::new(),
            orders: HashMap::new(),
        }
    }
}
