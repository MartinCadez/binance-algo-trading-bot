use binance_spot_connector_rust::{
    hyper::{BinanceHttpClient, Error},
    market::{self, klines::KlineInterval},
};
use tokio::time::{sleep, Duration};
use tokio_cron_scheduler::{Job, JobScheduler};

use crate::utils::objects::CandleStick;

// Delay between HTTP requests
const REQUEST_DELAY_MS: u64 = 250;

pub async fn fetch_hist_market_data(
    symbol: &'static str,
    limit: u32,
    interval: KlineInterval,
) -> Result<(), Error> {
    let client = BinanceHttpClient::default();

    println!("======= {} ========", interval);

    // Send request to the Binance server & wait for the response
    match client
        .send(market::klines(symbol, interval).limit(limit))
        .await
    {
        Ok(response) => {
            // Convert response to a String (AKA Serialization) if successful
            let data = response.into_body_str().await?; // Propagates I/O or encoding errors

            // Convert response to a JSON object (AKA Deserialization)
            if let Ok(klines) = serde_json::from_str::<Vec<serde_json::Value>>(&data) {
                // Print candlestick data
                for k in klines.iter().take(limit as usize) {
                    
                    let candlestick = CandleStick {
                        coin: symbol.to_string(),
                        open: k[1].as_str().unwrap_or("0.0").parse().unwrap_or(0.0),
                        high: k[2].as_str().unwrap_or("0.0").parse().unwrap_or(0.0),
                        low: k[3].as_str().unwrap_or("0.0").parse().unwrap_or(0.0),
                        close: k[4].as_str().unwrap_or("0.0").parse().unwrap_or(0.0),
                        volume: k[5].as_str().unwrap_or("0.0").parse().unwrap_or(0.0),
                        timestamp: k[0].as_i64().unwrap_or(0) as i32,
                    };
                    println!("{:#?}", candlestick);
                }
            }
        }
        Err(e) => println!("{}, {}, {}, {:?}", symbol, limit, interval, e),
    }

    // Avoid hitting the rate limit
    sleep(Duration::from_millis(REQUEST_DELAY_MS)).await;

    Ok(())
}

pub async fn scheduled_task(
    cron_expr: &str,
    symbol: &'static str,
    limit: u32,
    interval: KlineInterval,
) {
    let scheduler = JobScheduler::new().await.unwrap();

    // Schedule job to run at the specified cron expression
    scheduler
        .add(
            Job::new_async(cron_expr, {
                move |_uuid, _l| {
                    Box::pin(async move {
                        fetch_hist_market_data(symbol, limit, interval)
                            .await
                            .unwrap();
                    })
                }
            })
            .unwrap(),
        )
        .await
        .unwrap();

    // Schedule runner
    tokio::spawn(async move {
        scheduler.start().await.unwrap();
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_fetch_market_data_valid_intervals() {
        // Trade symbol pair
        let symbol: &'static str = "BTCUSDT";

        // Valid timeframes/intervals
        let intervals: Vec<KlineInterval> = vec![KlineInterval::Minutes1, KlineInterval::Minutes15];

        // Lookback for number of candlesticks, e.i. number of past klines of our interest
        let limit: u32 = 2;

        for i in intervals {
            let result = fetch_hist_market_data(symbol, limit, i).await;
            assert!(result.is_ok(), "{}", i);
        }
    }

    #[tokio::test]
    async fn test_scheduled_task() {
        let symbol: &'static str = "BTCUSDT";
        let intervals: Vec<KlineInterval> = vec![KlineInterval::Minutes1];
        let limit: u32 = 2;
        let cron_expr = "*/5 * * * * *";

        for interval in intervals {
            scheduled_task(cron_expr, symbol, limit, interval).await;
        }

        tokio::signal::ctrl_c()
            .await
            .expect("Failed to listen for Ctrl+C signal");
    }
}
