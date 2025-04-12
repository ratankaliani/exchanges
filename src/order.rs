/// Represents the side of an order - either a bid (buy) or ask (sell)
#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub enum Side {
    #[default]
    Bid,
    Ask,
}

/// Represents a single order in the orderbook
#[derive(Debug, Default, Clone)]
pub struct Order {
    pub id: OrderId,
    pub price: Price,
    pub quantity: Quantity,
    pub side: Side,
    pub account: String,
    pub timestamp: Timestamp,
}

impl Order {
    pub fn new(
        id: OrderId,
        price: Price,
        quantity: Quantity,
        side: Side,
        account: String,
        timestamp: Timestamp,
    ) -> Self {
        Self {
            id,
            price,
            quantity,
            side,
            account,
            timestamp,
        }
    }
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct OrderId(pub u64);

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Price(pub u64);

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct NegatedPrice(pub u64);

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Quantity(pub u64);

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Timestamp(pub u64);

impl From<Price> for NegatedPrice {
    fn from(price: Price) -> Self {
        NegatedPrice(u64::MAX - price.0)
    }
}

impl From<NegatedPrice> for Price {
    fn from(price: NegatedPrice) -> Self {
        Price(u64::MAX - price.0)
    }
}

// /// Create an order with the given parameters.
// ///
// /// # Example
// ///
// /// ```
// /// let order = order!(1, 100, 10, Side::Bid, "trader1", 1);
// /// ```
// ///
// /// # Arguments
// ///
// /// * `id` - The unique identifier for the order.
// /// * `price` - The price of the order.
// /// * `qty` - The quantity of the order.
// /// * `side` - The side of the order.
// /// * `account` - The account of the order.
// /// * `ts` - The timestamp of the order.
// #[macro_export]
// macro_rules! create_order {
//     ($id:expr, $price:expr, $qty:expr, $side:expr, $account:expr, $ts:expr) => {
//         crate::order::Order {
//             id: crate::order::OrderId($id),
//             price: crate::order::Price($price),
//             quantity: crate::order::Quantity($qty),
//             side: $side,
//             account: $account.to_string(),
//             timestamp: crate::order::Timestamp($ts),
//         }
//     };
// }
