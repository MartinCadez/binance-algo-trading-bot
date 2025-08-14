use crate::utils::objects::CandleStick;
use crate::utils::objects::TradeAction;
use crate::database_logic::db_crud::{is_position_open, open_trade, close_trade, get_open_trade_info};

use sqlx::PgPool;

/// ================================
/// SMA-based strategy + trade flow
/// ================================
/// This module:
/// 1) computes simple moving averages (SMA),
/// 2) derives a bullish/bearish signal from fast vs slow SMA,
/// 3) decides an action (EnterLong / ExitLong / Hold),
/// 4) executes the action by opening/closing a position in the DB.
///
/// Assumptions:
/// - `is_position_open` / `open_trade` / `close_trade` / `get_open_trade_info`
///   rely on a *position-based* trades schema (`symbol`, `status = 'OPEN'|'CLOSED'`, etc.).
/// - Candlestick timestamps must align with DB expectations (seconds for `to_timestamp()`).

/// Compute the simple moving average of the latest `lookback` candles.
/// Uses the *most recent* candles first (reverse iteration).
///
/// NOTE:
/// - If `lookback == 0`, this will divide by zero (panic).
/// - If `candlesticks.len() < lookback`, it will average fewer values
///   but still divide by `lookback` (biasing the result down). Consider
///   validating inputs before calling in production.
pub fn sma(
    candlesticks: &[CandleStick], 
    lookback: usize
) -> f64 {
    candlesticks
        .iter()
        .rev()          // latest candles first
        .take(lookback) // only the last `lookback` samples
        .map(|c| c.close)
        .sum::<f64>() / lookback as f64
}

/// Generate a trading signal:
/// - `true`  => bullish (fast SMA > slow SMA)
/// - `false` => bearish (fast SMA <= slow SMA)
///
/// Includes debug prints of closes and computed SMAs.
pub fn sma_signal(
    candlesticks: &[CandleStick],
    fast_lookback: usize,
    slow_lookback: usize
) -> bool {
    // Debug: print the closes vector
    println!("{:?}", candlesticks.iter().map(|c| c.close).collect::<Vec<f64>>());
    // Debug: print number of candlesticks
    println!{"Number of candlesticks: {}", candlesticks.len()};
    
    // Compute SMAs (see notes in `sma` about edge cases)
    let fast_ma = sma(candlesticks, fast_lookback);
    let slow_ma = sma(candlesticks, slow_lookback);

    println!(
        "Fast SMA: {:.2}, Slow SMA: {:.2}",
        fast_ma, slow_ma
    );
    
    fast_ma > slow_ma
}

/// Decide the next action based on:
/// - whether a position is already open in the DB,
/// - and the SMA-based bullish/bearish signal.
///
/// Returns one of:
/// - `EnterLong` (no open position & bullish),
/// - `ExitLong`  (open position & bearish),
/// - `Hold`      (otherwise).
pub async fn execute_trade_action(
    pool: &sqlx::PgPool,
    candlesticks: &[CandleStick],
    fast_period: usize,
    slow_period: usize,
    symbol: &str,
) -> Result<TradeAction, Box<dyn std::error::Error + Send + Sync>> {
    // Query DB: do we already have an OPEN position for this symbol?
    let has_open_position = is_position_open(pool, symbol).await?;

    // Compute the signal from SMAs over the provided candlesticks
    let is_bullish_signal = sma_signal(candlesticks, fast_period, slow_period);

    // Simple state machine
    match (has_open_position, is_bullish_signal) {
        (false, true) => Ok(TradeAction::EnterLong), // no open position & bullish signal -> Enter long
        (true, false) => Ok(TradeAction::ExitLong),  // open position & bearish signal -> Exit long
        _ => Ok(TradeAction::Hold),                  // otherwise -> Hold
    }
}

/// Perform the trade side-effects based on the decided action:
/// - On `EnterLong`: buys using the *entire* `current_balance` at the last close price,
///   stores the trade as OPEN in the DB.
/// - On `ExitLong`: fetches the open trade, computes PnL, closes it in the DB,
///   and adds PnL to `current_balance`.
/// - On `Hold`: prints and does nothing.
///
/// NOTE:
/// - `current_balance` is *not* decremented when opening a position. This is a
///   simplified accounting model where the cash balance remains constant during
///   the position and only PnL is applied on exit.
/// - Fees/slippage are not modeled.
/// - Ensure `last_candle.timestamp` unit matches the DB function `to_timestamp`
///   (seconds expected). If your timestamps are in milliseconds, you’d need to
///   convert before storing (not changed here).
pub async fn evaluate_decision(
    pool: &PgPool,
    candlesticks: &[CandleStick],
    current_balance: &mut f64,
    symbol: &str,
    fast_period: usize,
    slow_period: usize,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Use the most recent candle as the execution reference
    let last_candle = candlesticks.last().ok_or("No candlesticks available")?;

    // Decide action given state (DB) + signal (SMA crossover)
    match execute_trade_action(pool, candlesticks, fast_period, slow_period, symbol).await? {

        TradeAction::EnterLong => {
            // Use the entire balance to compute asset amount at the last close price
            let amount = *current_balance / last_candle.close;

            // Record an OPEN trade in the database
            open_trade(
                pool,
                symbol, 
                last_candle.close, 
                amount,
                *current_balance, // budget_used recorded as the whole balance
            ).await?;
            
            println!("[BUY] Opened long trade for {} at price {}", symbol, last_candle.close);
        }

        TradeAction::ExitLong => {
            // Close only if we can find an OPEN trade (guarded DB read)
            if let Some(open_trade) = get_open_trade_info(pool, symbol).await? {
                let exit_price = last_candle.close;

                // Plain PnL: (exit - entry) * amount
                // NOTE: ignores fees/slippage
                let pnl = (exit_price - open_trade.entry_price) * open_trade.amount;
                
                // Persist closure in DB with exit_time from candle timestamp
                close_trade(
                    pool, 
                    open_trade.id, 
                    exit_price, 
                    pnl, 
                    last_candle.timestamp as i64 // ensure this is seconds, not ms
                ).await?;
                
                println!("[SOLD] Closed long trade for {} at price {}, PnL: {}", symbol, exit_price, pnl);

                // Apply PnL to the balance (see note above on simplified accounting)
                *current_balance += pnl;
            } else {
                println!("No open trade to close");
            }
        }

        TradeAction::Hold => {
            println!("[NO ACTION] Holding position for {}", symbol);
        }
    }

    Ok(())
}
