use polars::prelude::*;
use std::ops::{Div, Sub, Mul};
use crate::utils::data_io::read_parquet;
use crate::utils::config::Settings;

pub fn run_backtest() -> PolarsResult<()> {

    // config load
    let backtest = Settings::load().expect("Failed to load settings").backtest;

    // config constants
    let parquet_path = backtest.parquet_path;
    let test_balance = backtest.test_balance;
    let fast_period = backtest.fast_period;
    let slow_period = backtest.slow_period;

    println!("Reading data from: {}", parquet_path);
    let df = read_parquet(&parquet_path)?
        .lazy()
        .select(
            [col("date"), col("close")]
        )
        .sort(vec!["date"], SortMultipleOptions {
            descending: vec![false],
            nulls_last: vec![true],
            multithreaded: true,
            maintain_order: false,
            limit: None,
        })
        .with_columns([
            (
                col("close")
                .div(
                    col("close")
                    .shift(lit(1))
                )
            )
            .alias("kline_return"),
        ])
        .with_columns([
            (
                    col("kline_return")
                    .cum_prod(false)
                    .mul(lit(test_balance))
            )
            .alias("benchmark_balance"),
        ])
        .with_columns([
            (
                col("benchmark_balance")
                .cum_max(false)
            )
            .alias("benchmark_peak"),
            (
                col("benchmark_balance")
                .sub(col("benchmark_balance")
                .cum_max(false))
            )
            .alias("benchmark_drawdown"),
            // (
            //     (
            //         col("benchmark_balance") - 
            //         col("benchmark_balance")
            //         .cum_max(false)
            //     ) / 
            //     (
            //         col("benchmark_balance")
            //         .cum_max(false)
            //     ) * lit(100.0)
            // )
            //     .alias("benchmark_drawdown_pct"),
            (
                col("close")
                .rolling_mean(RollingOptionsFixedWindow {
                    window_size: fast_period,
                    min_periods: 1,
                    weights: None,
                    center: false,
                    fn_params: None,
                })
            )
            .alias("fast_sma"),
            (
                col("close")
                .rolling_mean(RollingOptionsFixedWindow {
                    window_size: slow_period,
                    min_periods: 1,
                    weights: None,
                    center: false,
                    fn_params: None,
                })
            )
            .alias("slow_sma"),
        ])
        .with_columns([
            (
                col("fast_sma")
                .gt(col("slow_sma"))
            )
            .alias("in_position"),
        ])
        .with_columns([
            when(
                col("in_position")
                .shift(lit(1))
                .eq(lit(true))
            )
                .then(col("kline_return"))
                .otherwise(lit(1.0))
                .alias("strategy_return"),
        ])
        .with_columns([ 
            (
                col("strategy_return")
                .cum_prod(false)
                .mul(lit(test_balance))
            )
            .alias("strategy_balance"),
        ])
        .with_columns([
            (
                col("strategy_balance")
                .cum_max(false)
            )
            .alias("strategy_peak"),
        ])
        .with_columns([
            (
                col("strategy_balance")
                .sub(col("strategy_peak"))
            )
            .alias("strategy_drawdown"),
        ]) 
        .collect()?;

    let benchmark_balance = df
        .column("benchmark_balance")?
        .f64()?
        .tail(Some(1))
        .get(0)
        .expect("[ERROR] No values in `benchmark_balance`");
    
    let benchmark_drawdown = df
        .column("benchmark_drawdown")?
        .f64()?
        .div(
            df.column("benchmark_peak")?
                .f64()?
        )
        .min()
        .expect("[ERROR] No minimum value in `benchmark_drawdown`")
        * 100.0;

    let strategy_returns = df
        .column("strategy_balance")?
        .f64()?
        .tail(Some(1))
        .get(0)
        .expect("[ERROR] No values in `strategy_balance`");

    let timing_in_market = df.column("in_position")?
        .bool()?
        .sum()
        .expect("[ERROR] Failed to sum `in_position`") as f64
        / df.height() as f64
        * 100.0;

    let strategy_drawdown_series = df
        .column("strategy_drawdown")?
        .f64()?
        .div(df.column("strategy_peak")?.f64()?);

    let strategy_drawdown = strategy_drawdown_series
        .min()
        .expect("[ERROR] No minimum value in `strategy_drawdown`") * 100.0;

    println!("-----------------------------------------------------------");
    println!("SMA Crossover Strategy Backtest Analysis");
    println!("-----------------------------------------------------------");
    println!("[PARAMETER] Starting Balance: {:.2}$", test_balance);
    println!("[PARAMETER] Fast SMA: {}", fast_period);
    println!("[PARAMETER] Slow SMA: {}", slow_period);
    println!("-----------------------------------------------------------");
    println!("[BENCHMARK] Total Return: {:.0}$", benchmark_balance);
    println!("[BENCHMARK] PERFORMANCE: {:.2}%", (benchmark_balance / test_balance - 1.0) * 100.0);
    println!("[BENCHMARK] Drawdown: {:.2}%", benchmark_drawdown);
    println!("-----------------------------------------------------------");
    println!("[STRATEGY] Total Return: {:.0}$", strategy_returns);
    println!("[STRATEGY] PERFORMANCE: {:.2}%", (strategy_returns / test_balance - 1.0) * 100.0);
    println!("[STRATEGY] Drawdown: {:.2}%", strategy_drawdown);
    println!("[STRATEGY] Timing in Market: {:.2}%", timing_in_market);
    println!("-----------------------------------------------------------");


    // println!("{:?}", df.head(Some(10)));
    // println!("{:?}", df.tail(Some(10)));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run_backtest_execution() {
        let result = run_backtest();
        if let Err(e) = result {
            panic!("Backtest failed with error: {:?}", e);
        }
    }
}
