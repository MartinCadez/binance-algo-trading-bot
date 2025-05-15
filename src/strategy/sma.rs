pub fn calculate_sma(candles: &[CandleStick], period: usize) -> Vec<Option<f64>> {
    let mut sma_values = Vec::with_capacity(candles.len());

    for i in 0..candles.len() {
        if i + 1 < period {
            sma_values.push(None); // Not enough data yet
            continue;
        }

        let sum: f64 = candles[i + 1 - period..=i]
            .iter()
            .map(|c| c.close)
            .sum();

        let sma = sum / period as f64;
        sma_values.push(Some(sma));
    }

    sma_values
}