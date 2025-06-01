use chrono::{DateTime, Utc};

// #[derive(Debug, sqlx::FromRow)]
// pub struct Trade {
//     pub coin: String,
//     pub price: f64,
//     pub amount: f64,
//     pub timestamp: i64,
//     pub state: String, // either buy or sell
// }

// pub enum Signal {
//     Buy,
//     Sell,
//     Hold,
// }

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Trade {
    pub id: i64,
    pub symbol: String,
    pub entry_price: f64,
    pub exit_price: Option<f64>,
    pub amount: f64,
    pub budget_used: f64,
    pub pnl: Option<f64>,
    pub entry_time: DateTime<Utc>,
    pub exit_time: Option<DateTime<Utc>>,
    pub status: String,  // "OPEN" or "CLOSED"
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct CandleStick {
    pub coin: String,
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