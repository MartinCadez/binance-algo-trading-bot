use chrono::{DateTime, Utc};

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Trade {
    pub id: i64,
    #[allow(dead_code)]
    pub symbol: String,
    pub entry_price: f64,
    #[allow(dead_code)]
    pub exit_price: Option<f64>,
    pub amount: f64,
    #[allow(dead_code)]
    pub budget_used: f64,
    #[allow(dead_code)]
    pub pnl: Option<f64>,
    #[allow(dead_code)]
    pub entry_time: DateTime<Utc>,
    #[allow(dead_code)]
    pub exit_time: Option<DateTime<Utc>>,
    #[allow(dead_code)]
    pub status: String,  // "OPEN" or "CLOSED"
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct CandleStick {
    pub symbol: String,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
    pub timestamp: i64,
}

#[derive(Debug, PartialEq)]
pub enum TradeAction {
    EnterLong,
    ExitLong,
    Hold,
}