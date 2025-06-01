mod database_logic;
mod network;
mod strategy;
mod utils;

use crate::database_logic::{db_connect, db_crud};
use crate::network::api::market::{fetch_hist_market_data, scheduled_task};
use crate::strategy::sma::evaluate_decision;
use crate::utils::objects::CandleStick;

use binance_spot_connector_rust::market::klines::KlineInterval;
use dotenv::dotenv;
use std::env;
use tokio::sync::mpsc;

const LOOKBACK: u32 = 25;
const SYMBOL: &str = "BTCUSDT";
const TIMEFRAME: KlineInterval = KlineInterval::Minutes1;
const CRON_EXPRESSION: &str = "1 * * * * *"; // every minute

// Example values
const INITIAL_BALANCE: f64 = 500.0;
const FAST_PERIOD: usize = 10; 
const SLOW_PERIOD: usize = 25;


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set inside .env file");

    let pool = db_connect::connect_to_database(&database_url)
        .await
        .expect("Failed to connect to the database");
    
    db_crud::clear_trades_table(&pool)
        .await
        .expect("Failed to clear trades table");

    db_crud::clear_prices_table(&pool)
        .await
        .expect("Failed to clear trades table");

    let result = fetch_hist_market_data(SYMBOL, LOOKBACK, TIMEFRAME)
        .await
        .expect("Failed to fetch historical market data");

    db_crud::insert_prices(&pool, result)
        .await
        .expect("Failed to insert historical prices");

    // Open channel for receiving candlesticks
    let (tx, mut rx) = mpsc::channel::<Vec<CandleStick>>(2);

    // Task for fetching market data periodically
    scheduled_task(CRON_EXPRESSION, SYMBOL, SLOW_PERIOD as u32, TIMEFRAME, tx).await;

    // Spawn a task to process the received candlesticks
    tokio::spawn(async move {
    let mut current_balance = INITIAL_BALANCE;

    // Process incoming candlesticks
    while let Some(candlesticks) = rx.recv().await {
        let last_candle = candlesticks.last().unwrap();
        println!("Last candle: {:?}", last_candle);
        evaluate_decision(
            &pool,
            &candlesticks,
            &mut current_balance,
            SYMBOL,
            FAST_PERIOD,
            SLOW_PERIOD,
        ).await
        .expect("Failed to evaluate decision");
    }
    });
    tokio::signal::ctrl_c().await?;
    Ok(())
}
