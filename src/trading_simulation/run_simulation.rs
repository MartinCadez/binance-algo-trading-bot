use crate::trading_simulation::database::connection;
use crate::trading_simulation::network::api::market::spawn_cron_market_feed;
use crate::trading_simulation::strategy::sma_crossover::execute_trade_strategy;
use crate::trading_simulation::trade_analysis_report::generate_report;
use crate::utils::config::Settings;
use crate::utils::objects::CandleStick;

use dotenv::dotenv;
use std::env;
use tokio::sync::mpsc;

pub async fn run_trading_simulation() -> Result<(), Box<dyn std::error::Error>> {
    // config load
    let sim = Settings::load()
        .expect("Failed to load settings")
        .trading_simulation;

    sim.print_trading_simulation_params();

    // config constants
    let timeframe = sim
        .timeframe_as_binance()
        .expect("Invalid timeframe in config");
    let symbol = sim.symbol.clone();
    let initial_balance = sim.initial_balance;
    let fast_period = sim.fast_period;
    let slow_period = sim.slow_period;

    dotenv().ok(); // load env variables

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set inside .env file");

    let pool = connection::create_db_connection(&database_url)
        .await
        .expect("Connection to database failed");

    // atm market prices in db are not used in trading simulation since `lookback` is small
    // in case we would implement other trading strategies, that would rely on ML or some heavy stat anaysis
    // having those prices stored in db would come handy

    // start from a clean slate on each run
    // crud::clear_prices_table(&pool)
    //     .await
    //     .expect("Failed to clear prices table");

    // required data for initial input into `price` table
    // crud::insert_prices(
    //     &pool,
    //     fetch_market_data(symbol, slow_period, timeframe)
    //         .await
    //         .expect("Failed to fetch historical market data"),
    // )
    // .await
    // .expect("Failed to insert historical prices");

    // multi-producer single-consumer channel - multiple transmitors, only one receiver
    // channel capacity: one batch of candlestics (only vector can wait in channel)
    let (tx, mut rx) = mpsc::channel::<Vec<CandleStick>>(1);

    // periodically (each minute) fetch market data, aka cron process as tokio task
    // send batch candlesticks into channel
    spawn_cron_market_feed(symbol.clone(), slow_period, timeframe, tx).await;

    // trading execution task
    tokio::spawn(async move {
        // main processing lopp:
        // wait for incoming batch of candles from channel
        // and do trading part of simulation
        while let Some(candlesticks) = rx.recv().await {
            let mut current_balance = initial_balance;

            if candlesticks.is_empty() {
                continue;
            } // no trade if batch is empty

            // let last_candle = candlesticks.last().unwrap();
            // println!("Last candle: {:?}", last_candle);

            // for the  current tradim simulation configuration not needed
            // since current trading config strategy is not so computationally heavy
            // crud::add_price(&pool, last_candle.clone(), 100 as i32)
            //     .await
            //     .expect("Failed to insert price");

            // decision engine with db log, (buy/hold/sell)
            execute_trade_strategy(
                &pool,
                &candlesticks,
                &mut current_balance,
                &symbol,
                fast_period,
                slow_period,
            )
            .await
            .expect("Failed to evaluate decision");

            match generate_report(&pool, &symbol, initial_balance).await {
                Ok(report) => {
                    println!("{}", report.format_text());
                }
                Err(e) => eprintln!("Failed to generate report: {e}"),
            }
            
        }
    });

    tokio::signal::ctrl_c().await?;
    Ok(())
}
