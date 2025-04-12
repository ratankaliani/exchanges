use exchanges::{
    matching::MatchingEngine,
    order::{Order, OrderId, Price, Quantity, Side, Timestamp},
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
        let price = Price(rng.random_range(90..110));
        let quantity = Quantity(rng.random_range(1..100));

        let order = Order::new(
            OrderId(i),
            price,
            quantity,
            side,
            format!("trader{}", rng.random_range(1..100)),
            Timestamp(i),
        );

        engine.process_order(order);
    }

    let duration = start.elapsed();
    println!("Processed {} orders in {:?}", num_orders, duration);
    println!("Average time per order: {:?}", duration / num_orders as u32);
}
