mod database_logic;
mod network;
mod strategy;
mod utils;

use crate::database_logic::{db_connect, db_crud};
use crate::network::api::market::{fetch_hist_market_data, scheduled_task};
use crate::utils::objects::CandleStick;

use binance_spot_connector_rust::market::klines::KlineInterval;
use dotenv::dotenv;
use std::env;
use tokio::sync::mpsc;

const LOOKBACK: u32 = 200;
const SYMBOL: &str = "BTCUSDT";
const TIMEFRAME: KlineInterval = KlineInterval::Minutes1;
const CRON_EXPRESSION: &str = "0 * * * * *"; // every minute

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set inside .env file");

    let pool = db_connect::connect_to_database(&database_url)
        .await
        .expect("Failed to connect to the database");

    db_crud::clear_prices_table(&pool)
        .await
        .expect("Failed to clear trades table");

    let result = fetch_hist_market_data(SYMBOL, LOOKBACK, TIMEFRAME)
        .await
        .expect("Failed to fetch historical market data");

    db_crud::insert_prices(&pool, result)
        .await
        .expect("Failed to insert historical prices");

    let (tx, mut rx) = mpsc::channel::<Vec<CandleStick>>(2);

    scheduled_task(CRON_EXPRESSION, SYMBOL, 2, TIMEFRAME, tx).await;

    let db_pool = pool.clone();
    tokio::spawn(async move {
        while let Some(candlesticks) = rx.recv().await {
            for candle in candlesticks {
                match db_crud::add_price(&db_pool, candle, 1).await {
                    Ok(inserted) => println!("Inserted candlestick: {:?}", inserted),
                    Err(e) => eprintln!("Failed to insert candlestick: {:?}", e),
                }
            }
        }
    });

    tokio::signal::ctrl_c().await?;
    Ok(())
}
