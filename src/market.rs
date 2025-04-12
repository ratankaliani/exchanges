use crate::{asset::Asset, matching::MatchingEngine};

pub struct Market {
    pub numeraire: Asset,
    pub base: Asset,
    pub matching_engine: MatchingEngine,
}

impl Market {
    pub fn new(numeraire: Asset, base: Asset) -> Self {
        Market {
            numeraire,
            base,
            matching_engine: MatchingEngine::new(),
        }
    }
}
