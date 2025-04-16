// TODO objects (Column object, return objects, etc)

#[derive(Debug)]
pub struct Column {
    pub name: String,
    pub col_type: String,
    pub constraints: Option<String>, // Optional constraints like "NOT NULL", "PRIMARY KEY"
}

#[derive(Debug, sqlx::FromRow)]
pub struct Trade {
    pub id: i32,
    pub coin: String,
    pub amount: f64,
}

#[derive(Debug, sqlx::FromRow)]
pub struct Price {
    pub id: i32,
    pub open: f64,
    pub high:f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
    pub timestamp: String,
}