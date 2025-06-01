use binance_spot_connector_rust::{
    hyper::{BinanceHttpClient, Error},
    market::{self, klines::KlineInterval},
};
use tokio::sync::mpsc::Sender;
use tokio::time::{sleep, Duration};
use tokio_cron_scheduler::{Job, JobScheduler};

use crate::utils::objects::CandleStick;
use tokio::sync::oneshot;

// Delay between HTTP requests
const REQUEST_DELAY_MS: u64 = 250;

pub async fn fetch_hist_market_data(
    symbol: &'static str,
    limit: u32,
    interval: KlineInterval,
) -> Result<Vec<CandleStick>, Error> {
    let client = BinanceHttpClient::default();
    let mut candlesticks = Vec::new();
    let limit = limit + 1;

    match client
        .send(market::klines(symbol, interval).limit(limit))
        .await
    {
        Ok(response) => {
            let data = response.into_body_str().await?;
            if let Ok(klines) = serde_json::from_str::<Vec<serde_json::Value>>(&data) {
                for k in klines.iter().take(limit as usize) {
                    candlesticks.push(CandleStick {
                        coin: symbol.to_string(),
                        open: k[1].as_str().unwrap_or("0.0").parse().unwrap_or(0.0),
                        high: k[2].as_str().unwrap_or("0.0").parse().unwrap_or(0.0),
                        low: k[3].as_str().unwrap_or("0.0").parse().unwrap_or(0.0),
                        close: k[4].as_str().unwrap_or("0.0").parse().unwrap_or(0.0),
                        volume: k[5].as_str().unwrap_or("0.0").parse().unwrap_or(0.0),
                        timestamp: k[0].as_i64().unwrap_or(0),
                    });
                }

                // most recent candlestick is removed since its not closed yet
                candlesticks.pop();
            }
        }

        Err(e) => {
            println!("{}, {}, {}, {:?}", symbol, limit, interval, e);
            return Err(e);
        }
    }

    sleep(Duration::from_millis(REQUEST_DELAY_MS)).await;

    Ok(candlesticks)
}

pub async fn scheduled_task(
    cron_expr: &str,
    symbol: &'static str,
    limit: u32,
    interval: KlineInterval,
    tx: Sender<Vec<CandleStick>>,
) {
    let scheduler = JobScheduler::new().await.unwrap();

    scheduler
        .add(
            Job::new_async(cron_expr, {
                let tx = tx.clone();
                move |_uuid, _l| {
                    let tx = tx.clone();
                    Box::pin(async move {
                        match fetch_hist_market_data(symbol, limit - 1, interval).await {
                            Ok(candlesticks) => {
                                if let Err(err) = tx.send(candlesticks).await {
                                    eprintln!("Failed to send candlesticks: {}", err);
                                }
                            }
                            Err(e) => eprintln!("Error fetching market data: {:?}", e),
                        }
                    })
                }
            })
            .unwrap(),
        )
        .await
        .unwrap();

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
        let intervals: Vec<KlineInterval> = vec![KlineInterval::Minutes1];

        // Lookback for number of candlesticks, e.i. number of past klines of our interest
        let limit: u32 = 2;

        for i in intervals {
            let result = fetch_hist_market_data(symbol, limit, i).await;
            println!("{:?}", result.unwrap());
        }
    }

    #[tokio::test]
    async fn test_scheduled_task_with_output() {
        let (tx, mut rx) = tokio::sync::mpsc::channel::<Vec<CandleStick>>(10);

        scheduled_task("1 * * * * *", "BTCUSDT", 3, KlineInterval::Minutes1, tx).await;

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
