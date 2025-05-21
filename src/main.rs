mod database_logic;
mod network;
mod utils;
mod strategy;
use std::env;
use dotenv::dotenv;
use binance_spot_connector_rust::{
    isolated_margin_stream::new_listen_key, market::klines::KlineInterval
};
use tokio_tungstenite::tungstenite::protocol::frame::coding::Data;
use crate::utils::objects::{CandleStick, Signal};
use sqlx::PgPool;
use::sqlx::{Error};


#[tokio::main]
async fn main() {
    let symbol: &'static str = "BTCUSDT";
    let intervals: Vec<KlineInterval> = vec![KlineInterval::Minutes1];
    let limit: u32 = 5;
    let cron_expr = "*/5 * * * * *";
    let n = 200; 
    let budget: f64 = 200.0;
    // connect to database
    dotenv().ok(); // Load .env if present
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let pool = database_logic::db_connect::connect_to_database(&database_url).
    await.expect("");
    // create and clear bases
    database_logic::db_tables::create_prices_table(&pool).await.expect("");
    database_logic::db_tables::create_trades_table(&pool).await.expect("");
    database_logic::db_crud::clear_trades_table(&pool).await.expect("");

    // get last n data
    let result = network::api::market::fetch_hist_market_data(symbol, limit, KlineInterval::Minutes1).await.expect("");

    // get it on base
    database_logic::db_crud::insert_prices(&pool, result).await.expect("");

    // ====== start "loop" ========
/* 
    
    // get last candlestick
    let last_candlestick = network::api::market::scheduled_task(cron_expr, symbol, limit, interval).await;
    // add to database
    database_logic::db_crud::add_price(&pool, last_candlestick, n).await.expect("");
    // get from database
    let candles  = database_logic::db_crud::get_last_n_prices(&pool, n).await.expect("");
    // run strategy
    let short_period = 50;
    let long_period = 200;
    let signal = strategy::sma::generate_realtime_dual_sma_signal(&candles, short_period, long_period);
    // execute strategy
    match signal {
        Signal::Buy => budget = place_buy_order(&budget, &pool).await.expect(""),
        Signal::Sell => budget = place_sell_order(),
        Signal::Hold => println!("No action."),
    };
    
    // close all positions
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to listen for Ctrl+C signal");
}

async fn place_buy_order(budget: &f64, pool: &PgPool) -> Result<f64, Error>{
    // get last candlestick
    let candlesticks = database_logic::db_crud::get_last_n_prices(&pool, 1).await.expect("");
    // calculate amount
    let close = &candlesticks.first().unwrap().close;
    let amount_to_buy = budget / close;
    let trade = utils::objects::Trade{
        coin: candlesticks.first().unwrap().coin, // let trade accept references
        price: candlesticks.first().unwrap().close,
        amount: amount_to_buy,
        timestamp: candlesticks.first().unwrap().timestamp,
        state: "BUY".to_string(),
    };
    // add to database
    println!("We buy!");
    Ok(0.0)
}

fn place_sell_order() -> f64{
    // check if we own
    // if not just pass
    // else get last price and delete row
    println!("We sell!");
    return 0.0;
}
*/
}