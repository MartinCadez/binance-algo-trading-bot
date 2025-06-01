// Implement Create row, delete row, update row and read row or rows
use crate::utils::objects::CandleStick;
use crate::utils::objects::Trade;
use futures::TryFutureExt;
use ::sqlx::{Error, PgPool};

//===============TRADES=================
// add buy trade
// pub async fn add_buy_trade(pool: &PgPool, last_candlestick: &CandleStick, budget: f64) -> Result<(), sqlx::Error>{
//     let buy_trade = Trade{
//         coin: last_candlestick.coin.clone(),
//         price: last_candlestick.close,
//         amount: budget / last_candlestick.close, // calculate amount to buy
//         timestamp: last_candlestick.timestamp,
//         state: "BUY".to_string(),
//     };

//     add_trade(pool, buy_trade).await.expect("");
//     Ok(())
// }

// // add sell trade, return money we got
// pub async fn add_sell_trade(pool: &PgPool, last_candlestick: &CandleStick, budget: f64) -> Result<f64, sqlx::Error>{
    
//     let mut added_budget: f64 = 0.0;
//     // get last trade
//     if let Some(last_trade) = get_last_trade(pool).await? {
//         // Only add sell trade if the last trade was a "BUY"
//         if last_trade.state == "BUY" {
//             let sell_trade = Trade {
//                 coin: last_candlestick.coin.clone(),
//                 price: last_candlestick.close,
//                 amount: last_trade.amount,
//                 timestamp: last_candlestick.timestamp,
//                 state: "SELL".to_string(),
//             };
            
//             added_budget += last_trade.amount - last_candlestick.close;
//             add_trade(pool, sell_trade).await?; // propagate error if any
//         }
//     }
//     Ok(added_budget)
// }

// // add a trade to database trade table
// pub async fn add_trade(pool: &PgPool, trade: Trade) -> Result<Trade, sqlx::Error> {
//     let inserted_trade = sqlx::query_as::<_, Trade>(
//         "INSERT INTO trades (coin, price, amount, timestamp, state) VALUES ($1, $2, $3, $4, $5) RETURNING *"
//     )
//     .bind(trade.coin)
//     .bind(trade.price)
//     .bind(trade.amount)
//     .bind(trade.timestamp)
//     .bind(trade.state)
//     .fetch_one(pool)
//     .await?;

//     Ok(inserted_trade)
// }

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

pub async fn is_position_open(
    pool: &sqlx::PgPool,
    symbol: &str
) -> Result<bool, sqlx::Error> {
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

pub async fn open_trade(
    pool: &PgPool,
    symbol: &str,
    entry_price: f64,
    amount: f64,
    budget_used: f64,
    timestamp: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO trades 
        (symbol, entry_price, amount, budget_used, entry_time, status)
        VALUES ($1, $2, $3, $4, to_timestamp($5), 'OPEN')
        "#,
    )
    .bind(symbol)
    .bind(entry_price)
    .bind(amount)
    .bind(budget_used)
    .bind(timestamp)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn close_trade(
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
    symbol: &str
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