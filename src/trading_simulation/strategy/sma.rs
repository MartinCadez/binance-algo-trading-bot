use crate::trading_simulation::database::crud::{
    get_open_trade_info, is_position_open, record_close_trade, record_open_trade,
};
use crate::utils::objects::CandleStick;
use crate::utils::objects::TradeAction;

use sqlx::PgPool;

// simple moving average
pub fn sma(candlesticks: &[CandleStick], lookback: u32) -> f64 {
    candlesticks
        .iter()
        .rev() // latest candles first
        .take(lookback as usize) // only last `lookback` samples
        .map(|c| c.close)
        .sum::<f64>()
        / lookback as f64
}

// generating signals in trading strategy
pub fn sma_crossover(candlesticks: &[CandleStick], fast_lookback: u32, slow_lookback: u32) -> bool {
    // println!("{:?}", candlesticks.iter().map(|c| c.close).collect::<Vec<f64>>());
    // println!{"Number of candlesticks: {}", candlesticks.len()};

    let fast_ma = sma(candlesticks, fast_lookback);
    let slow_ma = sma(candlesticks, slow_lookback);

    println!("Fast SMA: {:.2}, Slow SMA: {:.2}", fast_ma, slow_ma);

    fast_ma > slow_ma
}

// determine action on each new candlestick based on configured `timeframe`
pub async fn get_trade_action(
    pool: &sqlx::PgPool,
    candlesticks: &[CandleStick],
    fast_period: u32,
    slow_period: u32,
    symbol: &str,
) -> Result<TradeAction, Box<dyn std::error::Error + Send + Sync>> {
    // query table `trades` in db for current trading position status
    let has_open_position: bool = is_position_open(pool, symbol).await?;

    let is_bullish_signal: bool = sma_crossover(candlesticks, fast_period, slow_period);

    // Formulation for sma-crossover strategy, can be modeled with;
    // -> Mealy machine, aka finite automata, deterministic FSM
    // states: S = {`no position open`, `position is open`}
    // initial state: S₀ = `no position open`
    // input alphabet: Σ = {bullish, bearish}
    // output alphabet: Λ = {`enter long`, `exit long`, `hold`}
    // transition function: T : S × Σ → S
    // output function:    G : S × Σ → Λ
    //
    // transition table (T):
    // -----------------------------------------------------
    // | current state      | input    | next state        |
    // -----------------------------------------------------
    // | no position open   | bullish  | position is open  |
    // | no position open   | bearish  | no position open  |
    // | position is open   | bullish  | position is open  |
    // | position is open   | bearish  | no position open  |
    // -----------------------------------------------------
    //
    // output table (G):
    // ----------------------------------------------
    // | current state      | input    | output      |
    // ----------------------------------------------
    // | no position open   | bullish  | enter long  |
    // | no position open   | bearish  | hold        |
    // | position is open   | bullish  | hold        |
    // | position is open   | bearish  | exit long   |
    // ---------------------------------------------
    match (has_open_position, is_bullish_signal) {
        (false, true) => Ok(TradeAction::EnterLong), // no open position & bullish signal -> Enter long
        (true, false) => Ok(TradeAction::ExitLong),  // open position & bearish signal -> Exit long
        _ => Ok(TradeAction::Hold),                  // otherwise -> Hold
    }
}

pub async fn execute_trade_strategy(
    pool: &PgPool,
    candlesticks: &[CandleStick],
    current_balance: &mut f64,
    symbol: &str,
    fast_period: u32,
    slow_period: u32,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let last_candle = candlesticks.last().ok_or("No candlesticks available")?;

    match get_trade_action(pool, candlesticks, fast_period, slow_period, symbol).await? {
        TradeAction::EnterLong => {
            let trade_size = *current_balance / last_candle.close;

            // db insert log
            record_open_trade(
                pool,
                symbol,
                last_candle.close,
                trade_size,
                *current_balance,
            )
            .await?;

            // assuming that there is no slippage or network latency, so
            // execution of trade happened at new candle open (current candle close)
            println!(
                "[BUY] Long trade open for {} at price {}",
                symbol, last_candle.close
            );
        }

        TradeAction::ExitLong => {
            // closing trade is possible only when position is open
            if let Some(open_trade) = get_open_trade_info(pool, symbol).await? {
                // assuming no slippage and no network latency
                let exit_price = last_candle.close;

                // asumming no fees
                let pnl = (exit_price - open_trade.entry_price) * open_trade.trade_size;

                // db insert log
                record_close_trade(
                    pool,
                    open_trade.id,
                    exit_price,
                    pnl,
                    last_candle.timestamp,
                )
                .await?;

                println!(
                    "[SOLD] Closed long trade for {} at price {}, PnL: {:.2}",
                    symbol, exit_price, pnl
                );

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
