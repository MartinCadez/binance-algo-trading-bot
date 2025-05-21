use crate::utils::objects::CandleStick;
use crate::utils::objects::Signal;

// calculates last sma
pub fn calculate_sma(candles: &[CandleStick], period: usize) -> Option<f64> {
    if candles.len() < period {
        return None;
    }

    let sum: f64 = candles[candles.len() - period..]
        .iter()
        .map(|c| c.close)
        .sum();

    Some(sum / period as f64)
}

// get sma signal
pub fn generate_realtime_dual_sma_signal(
    candles: &[CandleStick],
    short_period: usize,
    long_period: usize,
) -> Signal {
    if candles.len() < long_period + 1 {
        return Signal::Hold; // Not enough data
    }

    // Calculate previous and current short and long SMAs
    let prev_candles = &candles[..candles.len() - 1];
    let curr_candles = candles;

    let prev_short_sma = calculate_sma(prev_candles, short_period);
    let prev_long_sma = calculate_sma(prev_candles, long_period);

    let curr_short_sma = calculate_sma(curr_candles, short_period);
    let curr_long_sma = calculate_sma(curr_candles, long_period);

    match (prev_short_sma, prev_long_sma, curr_short_sma, curr_long_sma) {
        (Some(prev_s), Some(prev_l), Some(curr_s), Some(curr_l)) => {
            if prev_s < prev_l && curr_s > curr_l {
                Signal::Buy
            } else if prev_s > prev_l && curr_s < curr_l {
                Signal::Sell
            } else {
                Signal::Hold
            }
        }
        _ => Signal::Hold,
    }
}