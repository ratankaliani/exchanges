use exchanges::{
    matching::MatchingEngine,
    order::{AccountId, Order, OrderId, Price, Quantity, Side, Timestamp},
};
use rand::{Rng, rng};
use std::time::Instant;

fn main() {
    let mut engine = MatchingEngine::new();
    let mut rng = rng();
    let start = Instant::now();
    let num_orders = 100_000;

    // Generate and process random orders
    for i in 0..num_orders {
        let side = if rng.random_bool(0.5) {
            Side::Bid
        } else {
            Side::Ask
        };
        let price = Price::new(rng.random_range(90..110));
        let quantity = Quantity::new(rng.random_range(1..100));

        let order = Order::new(
            OrderId::new(i),
            price,
            quantity,
            side,
            AccountId::new(format!("trader{}", rng.random_range(1..100))),
            Timestamp::new(i),
        );

        engine.process_order(order);
    }

    let duration = start.elapsed();
    println!("Processed {} orders in {:?}", num_orders, duration);
    println!("Average time per order: {:?}", duration / num_orders as u32);
}
