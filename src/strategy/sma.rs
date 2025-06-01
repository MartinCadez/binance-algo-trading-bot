use crate::utils::objects::CandleStick;
use crate::utils::objects::TradeAction;
use crate::database_logic::db_crud::{is_position_open, open_trade, close_trade, get_open_trade_info};

use sqlx::PgPool;

pub fn sma(
    candlesticks: &[CandleStick], 
    lookback: usize
) -> f64 {
    candlesticks
        .iter() 
        .rev() // calculate average with latest candles first
        .take(lookback) 
        .map(|c| c.close)
        .sum::<f64>() / lookback as f64
}

pub fn sma_signal(
    candlesticks: &[CandleStick],
    fast_lookback: usize,
    slow_lookback: usize
) -> bool {
    println!("{:?}", candlesticks.iter().map(|c| c.close).collect::<Vec<f64>>());
    // print len of candlestick but just close data
    println!{"Number of candlesticks: {}", candlesticks.len()};
    
    
    let fast_ma = sma(candlesticks, fast_lookback);
    let slow_ma = sma(candlesticks, slow_lookback);

    println!(
        "Fast SMA: {:.2}, Slow SMA: {:.2}",
        fast_ma, slow_ma
    );
    
    fast_ma > slow_ma
}

pub async fn execute_trade_action(
    pool: &sqlx::PgPool,
    candlesticks: &[CandleStick],
    fast_period: usize,
    slow_period: usize,
    symbol: &str,
) -> Result<TradeAction, Box<dyn std::error::Error + Send + Sync>> {
    let has_open_position = is_position_open(pool, symbol).await?;
    let is_bullish_signal = sma_signal(candlesticks, fast_period, slow_period);

    match (has_open_position, is_bullish_signal) {
        (false, true) => Ok(TradeAction::EnterLong), // no open position & bullish signal -> Enter long
        (true, false) => Ok(TradeAction::ExitLong), // already open position & bearish signal -> Exit long
        _ => Ok(TradeAction::Hold) // already open position & bullish signal or no open position & bearish signal -> Hold
    }
}

pub async fn evaluate_decision(
    pool: &PgPool,
    candlesticks: &[CandleStick],
    current_balance: &mut f64,
    symbol: &str,
    fast_period: usize,
    slow_period: usize,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let last_candle = candlesticks.last().ok_or("No candlesticks available")?;

    match execute_trade_action(pool, candlesticks, fast_period, slow_period, symbol).await? {

        TradeAction::EnterLong => {
            let amount = *current_balance / last_candle.close; // asset amount based on current 
            open_trade(
                pool,
                symbol, 
                last_candle.close, 
                amount,
                *current_balance, 
            ).await?;
            
            println!("[BUY] Opened long trade for {} at price {}", symbol, last_candle.close);
        }

        TradeAction::ExitLong => {
            if let Some(open_trade) = get_open_trade_info(pool, symbol).await? {
                let exit_price = last_candle.close;
                let pnl = (exit_price - open_trade.entry_price) * open_trade.amount;
                
                close_trade(
                    pool, 
                    open_trade.id, 
                    exit_price, 
                    pnl, 
                    last_candle.timestamp as i64
                ).await?;
                
                println!("[SOLD] Closed long trade for {} at price {}, PnL: {}", symbol, exit_price, pnl);
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