use chrono::{DateTime, Utc};

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Trade {
    pub id: i64,
    #[allow(dead_code)]
    pub symbol: String,
    #[allow(dead_code)]
    pub entry_price: f64,
    #[allow(dead_code)]
    pub exit_price: Option<f64>,
    #[allow(dead_code)]
    pub trade_size: f64,
    #[allow(dead_code)]
    pub position_size: f64,
    #[allow(dead_code)]
    pub pnl: Option<f64>,
    #[allow(dead_code)]
    pub entry_time: DateTime<Utc>,
    #[allow(dead_code)]
    pub exit_time: Option<DateTime<Utc>>,
    #[allow(dead_code)]
    pub status: String, // `OPEN` or `CLOSED`
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct CandleStick {
    #[allow(dead_code)]
    pub symbol: String,
    #[allow(dead_code)]
    pub open: f64,
    #[allow(dead_code)]
    pub high: f64,
    #[allow(dead_code)]
    pub low: f64,
    #[allow(dead_code)]
    pub close: f64,
    #[allow(dead_code)]
    pub volume: f64,
    #[allow(dead_code)]
    pub timestamp: i64,
}

#[derive(Debug, PartialEq)]
pub enum TradeAction {
    EnterLong,
    ExitLong,
    Hold,
}
