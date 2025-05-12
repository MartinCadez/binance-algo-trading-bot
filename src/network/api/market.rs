use binance_spot_connector_rust::{
    hyper::{BinanceHttpClient, Error},
    market::{self, klines::KlineInterval},
};
use tokio::time::{sleep, Duration};

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
                    println!("{:#}", k);
                }
            }
        }
        Err(e) => println!("{}, {}, {}, {:?}", symbol, limit, interval, e),
    }

    // Avoid hitting the rate limit
    sleep(Duration::from_millis(REQUEST_DELAY_MS)).await;

    Ok(())
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
}
