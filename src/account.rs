use std::collections::HashMap;

use crate::asset::Asset;

pub struct Account {
    pub id: String,
    pub balances: HashMap<Asset, u64>,
}
