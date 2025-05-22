// Implement Create row, delete row, update row and read row or rows
use crate::utils::objects::CandleStick;
use crate::utils::objects::Trade;
use ::sqlx::{Error, PgPool};

//===============TRADES=================
// add a trade to database trade table
pub async fn add_trade(pool: &PgPool, trade: Trade) -> Result<Trade, sqlx::Error> {
    let inserted_trade = sqlx::query_as::<_, Trade>(
        "INSERT INTO trades (coin, price, amount, timestamp, state) VALUES ($1, $2, $3, $4, $5) RETURNING *"
    )
    .bind(trade.coin)
    .bind(trade.price)
    .bind(trade.amount)
    .bind(trade.timestamp)
    .bind(trade.state)
    .fetch_one(pool)
    .await?;

    Ok(inserted_trade)
}

// TODO: change to get last trade
pub async fn get_last_trade(pool: &PgPool) -> Result<Option<Trade>, sqlx::Error> {
    let query = "SELECT * FROM trades ORDER BY id DESC LIMIT 1";

    let result = sqlx::query_as::<_, Trade>(query)
        .fetch_optional(pool)
        .await?;

    Ok(result)
}

// Function for clearing trades table
pub async fn clear_trades_table(pool: &PgPool) -> Result<u64, sqlx::Error> {
    let result = sqlx::query("DELETE FROM trades").execute(pool).await?;

    Ok(result.rows_affected())
}

pub async fn clear_prices_table(pool: &PgPool) -> Result<u64, sqlx::Error> {
    let result = sqlx::query("DELETE FROM prices").execute(pool).await?;

    Ok(result.rows_affected())
}

//===============PRICES=================UNTESTED
// Create a row for our table and delete last if more than m
pub async fn add_price(
    pool: &PgPool,
    candle_stick: CandleStick,
    m: i32,
) -> Result<CandleStick, sqlx::Error> {
    // Insert new price
    let inserted = sqlx::query_as::<_, CandleStick>(
        "INSERT INTO prices (coin, open, high, low, close, volume, timestamp)
         VALUES ($1, $2, $3, $4, $5, $6, $7)
         RETURNING *",
    )
    .bind(candle_stick.coin)
    .bind(candle_stick.open)
    .bind(candle_stick.high)
    .bind(candle_stick.low)
    .bind(candle_stick.close)
    .bind(candle_stick.volume)
    .bind(candle_stick.timestamp)
    .fetch_one(pool)
    .await?;

    // Delete oldest price if count exceeds m
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM prices")
        .fetch_one(pool)
        .await?;

    if count.0 > m as i64 {
        // Delete the oldest row by id or timestamp
        sqlx::query(
            "DELETE FROM prices WHERE timestamp = (
                SELECT timestamp FROM prices ORDER BY timestamp ASC LIMIT 1)",
        )
        .execute(pool)
        .await?;
    }

    Ok(inserted)
}

// insert more prices in dataframe
pub async fn insert_prices(pool: &PgPool, candles: Vec<CandleStick>) -> Result<(), sqlx::Error> {
    // Use a transaction for performance and atomicity
    let mut tx = pool.begin().await?;

    for candle in candles {
        sqlx::query(
            "INSERT INTO prices (coin, open, high, low, close, volume, timestamp)
             VALUES ($1, $2, $3, $4, $5, $6, $7)",
        )
        .bind(candle.coin)
        .bind(candle.open)
        .bind(candle.high)
        .bind(candle.low)
        .bind(candle.close)
        .bind(candle.volume)
        .bind(candle.timestamp)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    Ok(())
}

// get last n prices
pub async fn get_last_n_prices(pool: &PgPool, n: i64) -> Result<Vec<CandleStick>, sqlx::Error> {
    let result = sqlx::query_as::<_, CandleStick>("SELECT * FROM prices ORDER BY id DESC LIMIT $1")
        .bind(n)
        .fetch_all(pool)
        .await?;

    Ok(result)
}

// function for clearing prices table
pub async fn delete_price(pool: &PgPool) -> Result<u64, sqlx::Error> {
    let result = sqlx::query("DELETE FROM prices").execute(pool).await?;

    Ok(result.rows_affected())
}
