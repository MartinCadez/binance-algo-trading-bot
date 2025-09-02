use binance_spot_connector_rust::market::klines::KlineInterval;
use config::{Config, File};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct TradingSimulation {
    pub symbol: String,
    pub timeframe: String,
    pub initial_balance: f64,
    pub fast_period: u32,
    pub slow_period: u32,
}

impl TradingSimulation {
    pub fn timeframe_as_binance(&self) -> Result<KlineInterval, String> {
        match self.timeframe.as_str() {
            "1m" => Ok(KlineInterval::Minutes1),
            "3m" => Ok(KlineInterval::Minutes3),
            "5m" => Ok(KlineInterval::Minutes5),
            "15m" => Ok(KlineInterval::Minutes15),
            "30m" => Ok(KlineInterval::Minutes30),
            "1h" => Ok(KlineInterval::Hours1),
            "2h" => Ok(KlineInterval::Hours2),
            "4h" => Ok(KlineInterval::Hours4),
            "6h" => Ok(KlineInterval::Hours6),
            "8h" => Ok(KlineInterval::Hours8),
            "12h" => Ok(KlineInterval::Hours12),
            "1d" => Ok(KlineInterval::Days1),
            "3d" => Ok(KlineInterval::Days3),
            "1w" => Ok(KlineInterval::Weeks1),
            other => Err(format!("Invalid timeframe: {}", other)),
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.initial_balance < 0.0 {
            return Err("Initial balance cannot be negative".into());
        }
        if self.fast_period < 1 || self.slow_period < 1 {
            return Err("SMA periods must be at least 1".into());
        }
        if self.fast_period > 10_000 || self.slow_period > 10_000 {
            return Err("SMA periods cannot exceed 10,000".into());
        }
        Ok(())
    }

    pub fn print_trading_simulation_params(&self) {
        println!("--- Trading Simulation Config ---");
        println!("Symbol          : {}", self.symbol);
        println!("Timeframe       : {}", self.timeframe);
        println!("Initial Balance : {}", self.initial_balance);
        println!("Fast SMA Period : {}", self.fast_period);
        println!("Slow SMA Period : {}", self.slow_period);
        println!("--------------------------------");
    }
}

#[derive(Debug, Deserialize)]
pub struct Backtest {
    pub parquet_path: String,
    pub test_balance: f64,
    pub fast_period: usize,
    pub slow_period: usize,
}

impl Backtest {
    pub fn validate(&self) -> Result<(), String> {
        if self.test_balance < 0.0 {
            return Err("Starting balance cannot be negative".into());
        }
        if self.fast_period < 2 || self.slow_period < 2 {
            return Err("Periods must be at least 2".into());
        }
        if self.fast_period > 10_000 || self.slow_period > 10_000 {
            return Err("Period cannot exceed 10,000".into());
        }
        Ok(())
    }
    
    pub fn print_backtest_params(&self) {
        println!("--- Backtest Config ---");
        println!("Parquet Path    : {}", self.parquet_path);
        println!("Test Balance    : {}", self.test_balance);
        println!("Fast SMA Period : {}", self.fast_period);
        println!("Slow SMA Period : {}", self.slow_period);
        println!("-----------------------");
    }
}

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub trading_simulation: TradingSimulation,
    pub backtest: Backtest,
}

impl Settings {
    pub fn load() -> Result<Self, String> {
        let settings: Settings = Config::builder()
            .add_source(File::with_name("config"))
            .build()
            .map_err(|e| format!("Failed to build config: {}", e))?
            .try_deserialize()
            .map_err(|e| format!("Failed to deserialize config: {}", e))?;

        settings.trading_simulation.validate()?;
        settings.backtest.validate()?;

        Ok(settings)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_config() {
        match Settings::load() {
            Ok(settings) => {
                println!("Loaded settings successfully: {:#?}", settings);
            }
            Err(err) => {
                println!("Error loading settings: {}", err);
            }
        }
    }
}
