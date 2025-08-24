mod database_logic;
mod network;
mod strategy;
mod utils;
mod analysis;
mod backtest;

use crate::database_logic::{db_connect, db_crud};
use crate::network::api::market::{fetch_hist_market_data, scheduled_task};
use crate::strategy::sma::evaluate_decision;
use crate::utils::objects::CandleStick;

use binance_spot_connector_rust::market::klines::KlineInterval;
use dotenv::dotenv;
use std::env;
use tokio::sync::mpsc;

const LOOKBACK: u32 = 25;                // number of candles to load initially
const SYMBOL: &str = "BTCUSDT";          // trading pair/symbol to track
const TIMEFRAME: KlineInterval = KlineInterval::Minutes1;
// 6-field cron (includes seconds): "sec min hour day month weekday"
// This runs at second 1 of every minute (e.g., HH:MM:01).
const CRON_EXPRESSION: &str = "1 * * * * *";

// Backtest / simulation params (tweak as needed)
const INITIAL_BALANCE: f64 = 500.0;
const FAST_PERIOD: usize = 10;
const SLOW_PERIOD: usize = 25;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load .env (e.g., DATABASE_URL) early
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set inside .env file");

    // Create a connection pool (cheap to clone; holds internal Arc)
    let pool = db_connect::connect_to_database(&database_url)
        .await
        .expect("Failed to connect to the database");

    // Clear prices table so we start from a clean slate on each run.
    db_crud::clear_prices_table(&pool)
        .await
        .expect("Failed to clear prices table");

    // Bootstrap: fetch historical candles to compute indicators (SMA, etc.)
    let result = fetch_hist_market_data(SYMBOL, LOOKBACK, TIMEFRAME)
        .await
        .expect("Failed to fetch historical market data");

    // Persist the historical candles (in a single transaction for speed)
    db_crud::insert_prices(&pool, result)
        .await
        .expect("Failed to insert historical prices");

    // Create a channel for incoming candle batches.
    // Small buffer (1) provides backpressure so we don't pile up work.
    let (tx, mut rx) = mpsc::channel::<Vec<CandleStick>>(1);

    // Schedule periodic market data fetches that send into `tx`.
    scheduled_task(CRON_EXPRESSION, SYMBOL, SLOW_PERIOD as u32, TIMEFRAME, tx).await;

    // Process incoming candle batches concurrently.
    // `pool` is moved into the task. If you also need it in main, clone first: let pool2 = pool.clone();
    tokio::spawn(async move {
        let mut current_balance = INITIAL_BALANCE;

        // Stream-processing loop; handles each emitted batch
        while let Some(candlesticks) = rx.recv().await {
            println!("Received candlesticks: {:?}", candlesticks.len());

            // SAFETY: unwrap() will panic if empty; if that’s possible, handle it:
            // if candlesticks.is_empty() { continue; }
            let last_candle = candlesticks.last().unwrap();
            println!("Last candle: {:?}", last_candle);

            // Core decision engine: reads/writes DB as needed and updates balance
            // (Make sure `evaluate_decision` is side-effect safe and handles errors)
            evaluate_decision(
                &pool,
                &candlesticks,
                &mut current_balance,
                SYMBOL,
                FAST_PERIOD,
                SLOW_PERIOD,
            )
            .await
            .expect("Failed to evaluate decision");

            match analysis::generate_report(&pool, SYMBOL, INITIAL_BALANCE).await {
                Ok(report) => {
                    println!("{}", report.format_text());   
                }
                Err(e) => eprintln!("Failed to generate report: {e}"),
            }
        }
    });

    // Keep the runtime alive until Ctrl+C. Ensures background tasks keep running.
    tokio::signal::ctrl_c().await?;
    Ok(())
}
