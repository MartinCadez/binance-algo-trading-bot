pub mod utils;
pub mod trading_simulation;
pub mod backtest;

use clap::{Parser, Subcommand};
use trading_simulation::run_trading_simulation;
use backtest::run_backtest;

#[derive(Parser)]
#[command(name = "bot")]
#[command(about = "trading bot CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Backtest,
    Trade,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Backtest => {
            if let Err(e) = run_backtest() {
                eprintln!("Backtest failed: {e}");
            }
        }
        Commands::Trade => {
            if let Err(e) = run_trading_simulation().await {
                eprintln!("Trading simulation failed: {e}");
            }
        }
    }
}
