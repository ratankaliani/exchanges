use anyhow::Result;
use exchanges::{
    asset::Asset,
    exchange::Exchange,
    market::{Market, Pair},
    order::{AccountId, Order, OrderId, Price, Quantity, Side, Timestamp},
};
use std::time::Instant;

fn main() -> Result<()> {
    // Create exchange and market
    let mut exchange = Exchange::new();
    let pair = Pair {
        numeraire: Asset::new("USD"),
        base: Asset::new("BTC"),
    };
    let market = Market::new(pair);
    exchange.add_market(market);

    // Create accounts and add balances
    let trader1 = AccountId::new("trader1".to_string());
    let trader2 = AccountId::new("trader2".to_string());
    let trader3 = AccountId::new("trader3".to_string());

    exchange
        .account_manager
        .add_balance(trader1.clone(), pair.numeraire, 500_000);
    exchange
        .account_manager
        .add_balance(trader1.clone(), pair.base, 5);
    exchange
        .account_manager
        .add_balance(trader2.clone(), pair.numeraire, 500_000);
    exchange
        .account_manager
        .add_balance(trader2.clone(), pair.base, 10);
    exchange
        .account_manager
        .add_balance(trader3.clone(), pair.numeraire, 100_000);
    exchange
        .account_manager
        .add_balance(trader3.clone(), pair.base, 5);

    let start = Instant::now();

    // Create and process some orders
    let orders = vec![
        Order::new(
            OrderId::new(1),
            Price::new(50_000),
            Quantity::new(2),
            Side::Ask,
            trader1.clone(),
            Timestamp::new(1),
        ),
        Order::new(
            OrderId::new(2),
            Price::new(49_000),
            Quantity::new(3),
            Side::Ask,
            trader3.clone(),
            Timestamp::new(2),
        ),
        Order::new(
            OrderId::new(3),
            Price::new(51_000),
            Quantity::new(4),
            Side::Bid,
            trader2.clone(),
            Timestamp::new(3),
        ),
    ];

    for order in orders {
        exchange.post_order(order, pair)?;
    }

    let duration = start.elapsed();
    println!("Processed orders in {:?}", duration);

    // Print final balances
    println!("\nFinal Balances:");
    println!("{:?}", exchange.account_manager.print_balances());
    Ok(())
}
