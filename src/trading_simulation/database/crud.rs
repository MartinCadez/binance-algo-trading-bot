use crate::utils::objects::Trade;
use ::sqlx::PgPool;

pub async fn is_position_open(pool: &sqlx::PgPool, symbol: &str) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar!(
        r#"
        SELECT EXISTS(
            SELECT 1 FROM trades 
            WHERE symbol = $1 AND status = 'OPEN'
        )"#,
        symbol
    )
    .fetch_one(pool)
    .await
    .map(|exists| exists.unwrap_or(false))
}

pub async fn record_open_trade(
    pool: &PgPool,
    symbol: &str,
    entry_price: f64,
    trade_size: f64,
    position_size: f64,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO trades 
        (symbol, entry_price, trade_size, position_size, entry_time, status)
        VALUES ($1, $2, $3, $4, CURRENT_TIMESTAMP, 'OPEN')
        "#,
    )
    .bind(symbol)
    .bind(entry_price)
    .bind(trade_size)
    .bind(position_size)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn record_close_trade(
    pool: &PgPool,
    trade_id: i64,
    exit_price: f64,
    pnl: f64,
    timestamp: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE trades
        SET 
            exit_price = $1,
            pnl = $2,
            exit_time = to_timestamp($3),
            status = 'CLOSED'
        WHERE id = $4
        "#,
    )
    .bind(exit_price)
    .bind(pnl)
    .bind(timestamp)
    .bind(trade_id)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_open_trade_info(
    pool: &PgPool,
    symbol: &str,
) -> Result<Option<Trade>, sqlx::Error> {
    sqlx::query_as!(
        Trade,
        r#"
        SELECT * 
        FROM trades
        WHERE symbol = $1 AND status = 'OPEN'
        LIMIT 1
        "#,
        symbol
    )
    .fetch_optional(pool)
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::trading_simulation::database::connection::create_db_connection;
    use chrono::Utc;
    use dotenv::dotenv;
    use std::env;

    #[tokio::test]
    async fn test_trade_log() {
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env");
        let symbol = "TESTSYM";

        dotenv().ok();

        let pool = create_db_connection(&database_url)
            .await
            .expect("Failed to connect to database");

        println!("[TEST] Using symbol: {}", symbol);

        let is_open = is_position_open(&pool, symbol).await.unwrap();
        println!("[TEST] Is position open initially? {}", is_open);
        assert_eq!(is_open, false, "No open trade should exist initially");

        println!("[TEST] Recording open trade...");
        record_open_trade(&pool, symbol, 100.0, 1.0, 0.5)
            .await
            .expect("Failed to insert open trade");

        let is_open = is_position_open(&pool, symbol).await.unwrap();
        println!("[TEST] Is position open after insert? {}", is_open);
        assert_eq!(is_open, true, "Open trade should exist after insert");

        let trade = get_open_trade_info(&pool, symbol)
            .await
            .expect("Failed to get open trade info")
            .expect("Open trade should exist");

        println!(
            "[TEST] Open trade info -> id: {}, symbol: {}, entry_price: {}, trade_size: {}, position_size: {}",
            trade.id, trade.symbol, trade.entry_price, trade.trade_size, trade.position_size
        );

        let trade_id = trade.id;
        let exit_price = 110.0;
        let pnl = exit_price - trade.entry_price;
        let timestamp = Utc::now().timestamp();

        println!(
            "[TEST] Closing trade id {} at exit price {}, PnL {:.2}",
            trade_id, exit_price, pnl
        );

        record_close_trade(&pool, trade_id, exit_price, pnl, timestamp)
            .await
            .expect("Failed to close trade");

        let is_open = is_position_open(&pool, symbol).await.unwrap();
        println!("[TEST] Is position open after closing? {}", is_open);
        assert_eq!(is_open, false, "Trade should be closed");

        println!("✅ All CRUD functions test passed for symbol {}", symbol);
    }
}

// #[allow(dead_code)]
// pub async fn get_last_trade(pool: &PgPool) -> Result<Option<Trade>, sqlx::Error> {
//     let query = "SELECT * FROM trades ORDER BY id DESC LIMIT 1";

//     let result = sqlx::query_as::<_, Trade>(query)
//         .fetch_optional(pool)
//         .await?;

//     Ok(result)
// }

// #[allow(dead_code)]
// // Function for clearing trades table
// pub async fn clear_trades_table(pool: &PgPool) -> Result<u64, sqlx::Error> {
//     let result = sqlx::query("DELETE FROM trades").execute(pool).await?;

//     Ok(result.rows_affected())
// }

// #[allow(dead_code)]
// // Function for clearing trades table
// pub async fn clear_prices_table(pool: &PgPool) -> Result<u64, sqlx::Error> {
//     let result = sqlx::query("DELETE FROM prices").execute(pool).await?;

//     Ok(result.rows_affected())
// }

// #[allow(dead_code)]
// pub async fn add_price(
//     pool: &PgPool,
//     candle_stick: CandleStick,
//     lookback: i32,
// ) -> Result<CandleStick, sqlx::Error> {
//     // Insert new price
//     let inserted = sqlx::query_as::<_, CandleStick>(
//         "INSERT INTO prices (coin, open, high, low, close, volume, timestamp)
//          VALUES ($1, $2, $3, $4, $5, $6, $7)
//          RETURNING *",
//     )
//     .bind(candle_stick.symbol)
//     .bind(candle_stick.open)
//     .bind(candle_stick.high)
//     .bind(candle_stick.low)
//     .bind(candle_stick.close)
//     .bind(candle_stick.volume)
//     .bind(candle_stick.timestamp)
//     .fetch_one(pool)
//     .await?;

//     let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM prices")
//         .fetch_one(pool)
//         .await?;

//     if count.0 > lookback as i64 {
//         // Delete the oldest row by id or timestamp
//         sqlx::query(
//             "DELETE FROM prices WHERE timestamp = (
//                 SELECT timestamp FROM prices ORDER BY timestamp ASC LIMIT 1)",
//         )
//         .execute(pool)
//         .await?;
//     }

//     Ok(inserted)
// }

// #[allow(dead_code)]
// pub async fn insert_prices(pool: &PgPool, candles: Vec<CandleStick>) -> Result<(), sqlx::Error> {
//     let mut tx = pool.begin().await?;

//     for candle in candles {
//         sqlx::query(
//             "INSERT INTO prices (coin, open, high, low, close, volume, timestamp)
//             VALUES ($1, $2, $3, $4, $5, $6, $7)",
//         )
//         .bind(candle.symbol)
//         .bind(candle.open)
//         .bind(candle.high)
//         .bind(candle.low)
//         .bind(candle.close)
//         .bind(candle.volume)
//         .bind(candle.timestamp)
//         .execute(&mut *tx)
//         .await?;
//     }

//     tx.commit().await?;
//     Ok(())
// }

// #[allow(dead_code)]
// pub async fn get_last_n_prices(pool: &PgPool, n: i64) -> Result<Vec<CandleStick>, sqlx::Error> {
//     let result = sqlx::query_as::<_, CandleStick>("SELECT * FROM prices ORDER BY id DESC LIMIT $1")
//         .bind(n)
//         .fetch_all(pool)
//         .await?;

//     Ok(result)
// }
