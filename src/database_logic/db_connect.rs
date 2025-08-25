use sqlx::PgPool;

// function for connecting to database, returns 'pool'
pub async fn connect_to_database(database_url: &String) -> Option<PgPool> {
    match PgPool::connect(database_url).await {
        Ok(pool) => {
            println!("✅[POSTGRES DB] Connected to database successfully");
            Some(pool)
        }
        Err(_) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dotenv::dotenv;
    use std::env;
    use crate::database_logic::db_crud;
    use crate::database_logic::db_connect;
    use crate::utils::objects::{CandleStick, Trade};

    #[tokio::test]
    async fn test_connection() {
        dotenv().ok(); // Load .env if present
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

        let pool = connect_to_database(&database_url).await;
        assert!(pool.is_some(), "Did not connect");
    }

    #[tokio::test]
    async fn test_connect_create_and_drop() {
        dotenv().ok(); // Load .env if present

        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let db_pool = db_connect::connect_to_database(&database_url).await.expect("Connection failed");
        
        // candlestick data object example, which is inserted trades table
        let market_data_example = CandleStick { 
            symbol: "BTC".to_string(),
            open: 5.5, 
            high: 5.7, 
            low: 5.2, 
            close: 5.3, 
            volume: 200.0,
            timestamp: 0,
        };
        
        // trade object example, which is inserted trades table
        let trade_example = Trade { 
            id: 1,
            symbol: "BTC".to_string(),
            entry_price: 100_000.0,
            exit_price: None,
            amount: 0.1,
            budget_used: 10_000.0,
            pnl: None,
            entry_time: chrono::Utc::now(),
            exit_time: None,
            status: "OPEN".to_string(),
        };
        
        // add candlestick object example into prices table
        db_crud::add_price(
            &db_pool,
            market_data_example,
            3
        ).await.expect("Failed to add price");

        // add trade object example into trades table

        db_crud::open_trade(
            &db_pool,
            &trade_example.symbol,
            trade_example.entry_price,
            trade_example.amount,
            trade_example.budget_used,
        ).await.expect("Failed to open trade");

        let last_trade = db_crud::get_last_trade(&db_pool).await.expect("Failed to load last trade");
        println!("Last Trade: {:?}", last_trade);
        
        let last_3_prices = db_crud::get_last_n_prices(&db_pool, 3).await.expect("Failed to load prices");

        println!("Prices: {:?}", last_3_prices);
        
    }
}