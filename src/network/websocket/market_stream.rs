use anyhow::{anyhow, Result};
use futures::{SinkExt, StreamExt};
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use tokio_tungstenite::{
    connect_async,
    tungstenite::{http::Uri, Message},
};

const PING_INTERVAL: Duration = Duration::from_secs(180);

#[derive(Debug, Clone, Deserialize)]
pub struct KlineData {
    #[serde(rename = "t")]
    pub open_time: i64,
    #[serde(rename = "T")]
    pub close_time: i64,
    #[serde(rename = "s")]
    pub symbol: String,
    #[serde(rename = "i")]
    pub interval: String,
    #[serde(rename = "f")]
    pub first_trade_id: i64,
    #[serde(rename = "L")]
    pub last_trade_id: i64,
    #[serde(rename = "o")]
    pub open: String,
    #[serde(rename = "c")]
    pub close: String,
    #[serde(rename = "h")]
    pub high: String,
    #[serde(rename = "l")]
    pub low: String,
    #[serde(rename = "v")]
    pub volume: String,
    #[serde(rename = "n")]
    pub number_of_trades: i64,
    #[serde(rename = "x")]
    pub is_closed: bool,
    #[serde(rename = "q")]
    pub quote_asset_volume: String,
    #[serde(rename = "V")]
    pub taker_buy_base_volume: String,
    #[serde(rename = "Q")]
    pub taker_buy_quote_volume: String,
    #[serde(rename = "B")]
    pub ignore_field: String,
}

#[derive(Debug, Deserialize)]
pub struct KlineEvent {
    #[serde(rename = "e")]
    pub event_type: String,
    #[serde(rename = "E")]
    pub event_time: i64,
    #[serde(rename = "s")]
    pub symbol: String,
    #[serde(rename = "k")]
    pub kline: KlineData,
}

#[derive(Debug, Deserialize)]
pub struct WsKlineEvent {
    pub stream: String,
    pub data: KlineEvent,
}

#[derive(Debug, Clone)]
pub struct Candle {
    pub symbol: String,
    pub timeframe: String,
    pub open: f64,
    pub close: f64,
    pub high: f64,
    pub low: f64,
    pub volume: f64,
    pub quote_volume: f64,
    pub taker_buy_base_volume: f64,
    pub taker_buy_quote_volume: f64,
    pub open_time: i64,
    pub close_time: i64,
    pub is_closed: bool,
    pub number_of_trades: i64,
}

impl From<&KlineData> for Candle {
    fn from(kline: &KlineData) -> Self {
        Candle {
            symbol: kline.symbol.clone(),
            timeframe: kline.interval.clone(),
            open: kline.open.parse().unwrap_or(0.0),
            close: kline.close.parse().unwrap_or(0.0),
            high: kline.high.parse().unwrap_or(0.0),
            low: kline.low.parse().unwrap_or(0.0),
            volume: kline.volume.parse().unwrap_or(0.0),
            quote_volume: kline.quote_asset_volume.parse().unwrap_or(0.0),
            taker_buy_base_volume: kline.taker_buy_base_volume.parse().unwrap_or(0.0),
            taker_buy_quote_volume: kline.taker_buy_quote_volume.parse().unwrap_or(0.0),
            open_time: kline.open_time,
            close_time: kline.close_time,
            is_closed: kline.is_closed,
            number_of_trades: kline.number_of_trades,
        }
    }
}

fn get_ws_klinedata_url(base_url: &str, symbol: &str, timeframes: &[&str]) -> String {
    format!(
        "{}/stream?streams={}",
        base_url,
        timeframes
            .iter()
            .map(|tf| format!("{}@kline_{}", symbol.to_lowercase(), tf))
            .collect::<Vec<_>>()
            .join("/")
    )
}

async fn process_message(message: Message, candle_tx: &mpsc::Sender<Candle>) -> Result<()> {
    match message {
        Message::Text(text) => {
            println!("Raw message: {}", text);

            let value: Value = serde_json::from_str(&text)?;

            let stream = value
                .get("stream")
                .and_then(|s| s.as_str())
                .unwrap_or_default();

            if !stream.contains("kline_") {
                return Ok(());
            }

            let kline_event: WsKlineEvent = serde_json::from_str(&text)?;

            let candle = Candle::from(&kline_event.data.kline);

            candle_tx
                .send(candle)
                .await
                .map_err(|e| anyhow!("Failed to send candle: {}", e))?;
        }

        Message::Close(_) => {
            return Err(anyhow!("Connection closed by server"));
        }

        Message::Ping(_) => {
            println!("Received ping, sending pong");
        }

        Message::Pong(_) => {
            println!("Received pong");
        }

        // ignoring other messages
        _ => {}
    }

    Ok(())
}

async fn manage_connection(url: String, candle_tx: mpsc::Sender<Candle>) -> Result<()> {
    let uri = url.parse::<Uri>()?;
    let (ws_stream, _response) = connect_async(uri).await?;
    let (mut write, mut read) = ws_stream.split();
    let mut last_ping = Instant::now();

    while let Some(message) = read.next().await.transpose()? {
        if last_ping.elapsed() > PING_INTERVAL {
            write.send(Message::Pong(vec![].into())).await?;
            last_ping = Instant::now();
        }

        process_message(message, &candle_tx).await?;
    }

    Ok(())
}

async fn process_candles(mut candle_rx: mpsc::Receiver<Candle>) {
    let mut candle_store: HashMap<String, Vec<Candle>> = HashMap::new();

    while let Some(candle) = candle_rx.recv().await {
        let timeframe_candles = candle_store.entry(candle.timeframe.clone()).or_default();
        timeframe_candles.push(candle.clone());

        if timeframe_candles.len() > 1000 {
            timeframe_candles.remove(0);
        }

        println!(
            "[{}] {} | O: {:.2} | H: {:.2} | L: {:.2} | C: {:.2} | V: {:.4} | QV: {:.2} | Trades: {} | {}",
            candle.timeframe,
            candle.symbol,
            candle.open,
            candle.high,
            candle.low,
            candle.close,
            candle.volume,
            candle.quote_volume,
            candle.number_of_trades,
            if candle.is_closed { "CLOSED" } else { "OPEN" }
        );
    }
}

/* 
#[cfg(test)]
mod integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_ws_conn() -> Result<()> {
        let ws_base_url = "wss://stream.binance.com:9443";
        let default_symbol = "BTCUSDT";
        let timeframes = &["1m", "3m", "15m"];

        let (candle_tx, candle_rx) = tokio::sync::mpsc::channel(1000);

        let ws_url = get_ws_klinedata_url(ws_base_url, default_symbol, timeframes);
        println!("WS URL: {}", ws_url);

        tokio::spawn(process_candles(candle_rx));

        match manage_connection(ws_url, candle_tx).await {
            Ok(_) => {}
            Err(e) => {
                eprintln!("{:?}", e);
            }
        }

        Ok(())
    }
}
    */
