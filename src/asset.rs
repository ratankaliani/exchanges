#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Asset {
    pub symbol: &'static str,
}

impl Asset {
    pub fn new(symbol: &'static str) -> Self {
        Self { symbol }
    }
}
