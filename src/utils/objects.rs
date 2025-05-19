#[derive(Debug, sqlx::FromRow)]
pub struct Trade {
    pub coin: String,
    pub price: f64,
    pub amount: f64,
    pub timestamp: i32,
    pub state: String, // either buy or sell
}

#[derive(Debug, sqlx::FromRow)]
pub struct CandleStick {
    pub coin: String,
    pub open: f64,
    pub high:f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
    pub timestamp: i32,
}