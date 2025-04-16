// Implement Create row, delete row, update row and read row or rows
use sqlx::PgPool;
use super::db_objects::Trade;
use super::db_objects::Price;


//===============TRADES=================
// Create a row for our table
pub async fn create_trade(pool: &PgPool, name: &str, amount: f64) -> Result<Trade, sqlx::Error> {
    let user = sqlx::query_as::<_, Trade>(
        "INSERT INTO Trades (coin, amount) VALUES ($1, $2) RETURNING *"
    )
    .bind(name)
    .bind(amount)
    .fetch_one(pool)
    .await?;

    Ok(user)
}

pub async fn get_trade_by_id(pool: &PgPool, user_id: i32) -> Result<Option<Trade>, sqlx::Error> {
    let result = sqlx::query_as::<_, Trade>("SELECT * FROM trades WHERE id = $1")
        .bind(user_id)
        .fetch_optional(pool)
        .await?;

    Ok(result)
}

pub async fn delete_trade(pool: &PgPool, user_id: i32) -> Result<u64, sqlx::Error> {
    let result = sqlx::query("DELETE FROM trades WHERE id = $1")
        .bind(user_id)
        .execute(pool)
        .await?;

    Ok(result.rows_affected())
}

//===============PRICES=================UNTESTED
// Create a row for our table
pub async fn create_price(pool: &PgPool, id: &str, open: f64, high: f64, low: f64, close: f64, volume: f64, timestamp: String) -> Result<Price, sqlx::Error> {
    let user = sqlx::query_as::<_, Price>(
        "INSERT INTO prices (id, open, high, low, close, volume, timestamp) VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING *"
    )
    .bind(id)
    .bind(open)
    .bind(high)
    .bind(low)
    .bind(close)
    .bind(volume)
    .bind(timestamp)
    .fetch_one(pool)
    .await?;

    Ok(user)
}

pub async fn get_price_by_id(pool: &PgPool, user_id: i32) -> Result<Option<Price>, sqlx::Error> {
    let result = sqlx::query_as::<_, Price>("SELECT * FROM prices WHERE id = $1")
        .bind(user_id)
        .fetch_optional(pool)
        .await?;

    Ok(result)
}

pub async fn delete_price(pool: &PgPool, user_id: i32) -> Result<u64, sqlx::Error> {
    let result = sqlx::query("DELETE FROM prices WHERE id = $1")
        .bind(user_id)
        .execute(pool)
        .await?;

    Ok(result.rows_affected())
}