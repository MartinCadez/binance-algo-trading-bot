use binance_spot_connector_rust::{
    hyper::{BinanceHttpClient, Error},
    market::{self, klines::KlineInterval},
};
use tokio::sync::mpsc::Sender;
use tokio::time::{Duration, sleep};
use tokio_cron_scheduler::{Job, JobScheduler};

use crate::utils::objects::CandleStick;

const CRON_EXPRESSION: &str = "1 * * * * *"; // each minute at first second
const REQUEST_DELAY_MS: u64 = 250; // Binance API constrain

pub async fn fetch_market_data(
    symbol: String,
    lookback: u32,
    timeframe: KlineInterval,
) -> Result<Vec<CandleStick>, Error> {
    let client = BinanceHttpClient::default();
    let mut candlesticks = Vec::new();

    // request one extra candlestick because the latest one (candlestick with index 0) is still open
    // for discrete trading strategies, only closed candles are reliable
    let lookback = lookback + 1;

    // sending http request asynchronously
    match client
        .send(market::klines(&symbol, timeframe).limit(lookback))
        .await
    {
        Ok(response) => {
            let data = response.into_body_str().await?; // read JSON object from http response body

            // try to parse JSON object to serde object
            if let Ok(klines) = serde_json::from_str::<Vec<serde_json::Value>>(&data) {
                for k in klines.iter().take(lookback as usize) {
                    candlesticks.push(CandleStick {
                        symbol: symbol.to_string(),
                        open: k[1].as_str().unwrap().parse::<f64>().unwrap(),
                        high: k[2].as_str().unwrap().parse::<f64>().unwrap(),
                        low: k[3].as_str().unwrap().parse::<f64>().unwrap(),
                        close: k[4].as_str().unwrap().parse::<f64>().unwrap(),
                        volume: k[5].as_str().unwrap().parse::<f64>().unwrap(),
                        timestamp: k[0].as_i64().unwrap(),
                    });
                }

                // most recent candlestick is removed since its not closed yet
                candlesticks.pop();
            }
        }

        Err(e) => {
            println!("{}, {}, {}, {:?}", symbol, lookback, timeframe, e);
            return Err(e);
        }
    }

    sleep(Duration::from_millis(REQUEST_DELAY_MS)).await; // Binance API constraint

    Ok(candlesticks)
}


// periodically fetch market candlestick data and send it to async channel 
// to be consumed by main trading async task
pub async fn spawn_cron_market_feed(
    symbol: String,
    lookback: u32,
    timeframe: KlineInterval,
    tx: Sender<Vec<CandleStick>>,
) {
    // cron scheduler
    let scheduler = JobScheduler::new()
        .await
        .unwrap();

    // task register
    scheduler
        .add(
            // create cron job
            Job::new_async(
            CRON_EXPRESSION,
            {

                // lifetime: until scheduler is not terminated
                move |_uuid, _l| {
                    
                    // lifetime: one cron execution
                    // fresh ownership for each closure execution (tokio stuff)
                    let tx = tx.clone();
                    let symbol = symbol.clone();

                    // keeping consistent adress in virtual memory 
                    // pinning prevents movement during .await suspension (tokio stuff)
                    Box::pin(

                        // move local vars to anonymous struct made by compiler
                        // with local vars and current state, future trait is implemented
                        // pool enum tells executor when data is ready or not, to proceed
                        async move {
                            match fetch_market_data(symbol, lookback, timeframe).await {

                                // send data to channel
                                Ok(candlesticks) => {
                                    if let Err(err) = tx.send(candlesticks).await {
                                        eprintln!("Failed to send candlesticks: {}", err);
                                    }
                                }
                                Err(e) => eprintln!("Error fetching market data: {:?}", e),
                            }
                        }
                    )
                }

            })
            .unwrap(),
        )
        .await
        .unwrap();

    // spawn task
    tokio::spawn(async move {
        scheduler.start().await.unwrap();
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_hist_data_fetch() {
        let symbol: String = "BTCUSDT".to_string();
        let timeframes: Vec<KlineInterval> = vec![KlineInterval::Minutes1, KlineInterval::Minutes3];
        let lookback: u32 = 2;

        for i in timeframes {
            let result = fetch_market_data(symbol.clone(), lookback, i).await;
            println!("{:?}", result.unwrap());
        }
    }

    #[tokio::test]
    async fn test_scheduled_task_with_output() {
        let symbol: String = "BTCUSDT".to_string();
        let timeframe: KlineInterval = KlineInterval::Minutes1;
        let lookback: u32 = 3;
        let (tx, mut rx) = tokio::sync::mpsc::channel::<Vec<CandleStick>>(10);

        spawn_cron_market_feed(symbol.clone(), lookback, timeframe, tx).await;

        tokio::spawn(async move {
            while let Some(candles) = rx.recv().await {
                println!("Received candlesticks: {:?}", candles);
            }
        });

        tokio::signal::ctrl_c()
            .await
            .expect("Failed to listen for Ctrl+C signal");
    }
}
