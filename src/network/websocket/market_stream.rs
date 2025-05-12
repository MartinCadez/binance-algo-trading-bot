use futures::{SinkExt, StreamExt};
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::http::Uri;
use tokio_tungstenite::{connect_async, tungstenite::Message};

const BINANCE_FUTURES_WS_BASE_URL: &str = "wss://stream.binance.com:9443";
// const DEFAULT_SYMBOL: &str = "BTCUSDT";
// const TIMEFRAMES: &[&str] = &["1m", "3m", "5m", "15m"];

#[derive(Debug)]
pub enum MarketDataError {
    ConnectionError(String),
    ParseError(String),
    WebSocketError(String),
    SubscriptionError(String),
    ChannelError(String),
}

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

fn create_kline_ws_url(symbol: &str, timeframes: &[&str]) -> Result<String, MarketDataError> {
    // Binance WebSocket API endpoint
    let endpoint = format!(
        "{}/stream?streams={}",
        BINANCE_FUTURES_WS_BASE_URL,
        timeframes
            .iter()
            .map(|tf| format!("{}@kline_{}", symbol.to_lowercase(), tf))
            .collect::<Vec<_>>()
            .join("/")
    );

    return Ok(endpoint);
}

async fn process_message(
    message: Message,                 // Raw ws message
    candle_tx: &mpsc::Sender<Candle>, // Channel to send processed candles
) -> Result<(), MarketDataError> {
    match message {
        Message::Text(text) => {
            // println!("{}", text);

            // Deserialize the message into a KlineEvent
            if let Ok(kline_event) = serde_json::from_str::<WsKlineEvent>(&text)
                .map_err(|e| MarketDataError::ParseError(format!("Kline parse error: {}", e)))
                .and_then(|event| {
                    if event.stream.contains("kline_") {
                        Ok(event)
                    } else {
                        Err(MarketDataError::ParseError("Non-kline event".into()))
                    }
                })
            {
                candle_tx
                    .send(Candle::from(&kline_event.data.kline))
                    .await
                    .map_err(|e| {
                        MarketDataError::ChannelError(format!("Failed to send candle: {}", e))
                    })?;
            }
        }
        Message::Close(_) => {
            return Err(MarketDataError::WebSocketError(
                "Connection closed by server".into(),
            ));
        }
        _ => {}
    }
    Ok(())
}

async fn manage_websocket_connection(
    url: String,
    candle_tx: mpsc::Sender<Candle>,
) -> Result<(), MarketDataError> {
    let uri = url
        .parse::<Uri>()
        .map_err(|e| MarketDataError::ConnectionError(format!("Failed to parse URL: {}", e)))?;

    println!("Connecting to WebSocket at: {}", uri);

    let (ws_stream, response) = connect_async(uri)
        .await
        .map_err(|e| MarketDataError::ConnectionError(format!("Failed to connect: {}", e)))?;

    println!("Connected to WebSocket. HTTP status: {}", response.status());
    println!("Response headers: {:?}", response.headers());

    let (mut write, mut read) = ws_stream.split();

    while let Some(message) = read.next().await {
        match message {
            Ok(msg) => match &msg {
                Message::Ping(data) => {
                    println!("Received ping from server");
                    write.send(Message::Pong(data.clone())).await.map_err(|e| {
                        MarketDataError::WebSocketError(format!("Failed to send pong: {}", e))
                    })?;
                }
                Message::Pong(_) => {
                    println!("Received pong from server");
                }
                _ => {
                    if let Err(e) = process_message(msg, &candle_tx).await {
                        println!("Error processing message: {:?}", e);
                        break;
                    }
                }
            },
            Err(e) => {
                return Err(MarketDataError::WebSocketError(format!(
                    "WebSocket error: {}",
                    e
                )));
            }
        }
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
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    const SYMBOL: &str = "btcusdt";

    #[tokio::test]
    async fn test_websocket_connection() -> Result<(), MarketDataError> {
        let timeframes: &[&str] = &["1m"];
        let (candle_tx, candle_rx) = mpsc::channel(1000);
        let ws_url = create_kline_ws_url(SYMBOL, timeframes)?;
        println!("WebSocket URL: {}", ws_url);
        let processor_handle = tokio::spawn(process_candles(candle_rx));
        let result = manage_websocket_connection(ws_url, candle_tx).await;
        processor_handle.await.map_err(|e| {
            MarketDataError::ChannelError(format!("Processor task failed: {:?}", e))
        })?;
        result?;
        Ok(())
    }
}
