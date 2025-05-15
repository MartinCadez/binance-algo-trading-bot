use sqlx::PgPool;

pub async fn connect_to_database(database_url: &String) -> Option<PgPool> {
    //println!("Connecting to database at {}", database_url);
    let pool = match PgPool::connect(database_url).await {
        Ok(pool) => {
            println!(" Successfully connected to the database!");
            Some(pool) // Return the pool if connection is successful
        }
        Err(_e) => {
            None // Return None if connection fails
        }
    };
    return pool;
}

#[cfg(test)]
mod tests {
    use super::*;
    use dotenv::dotenv;
    use std::env;
    use crate::database_logic::db_tables;
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
        
        // create pool
        let pool = db_connect::connect_to_database(&database_url).await.expect("Failed connecting");
        
        // create tables
        db_tables::create_prices_table(&pool).await.expect("Failed to create prices table");
        db_tables::create_trades_table(&pool).await.expect("Failed to create trades table");
        
        let price = CandleStick { 
            coin: "BTC".to_string(),
            open: 5.5, 
            high: 5.7, 
            low: 5.2, 
            close: 5.3, 
            volume: 200.0,
            timestamp: 0,
        };
        let trade = Trade { 
            coin: "BTC".to_string(), 
            price: 5.4, 
            amount: 100.0, 
            timestamp: 0, 
            state: "BUY".to_string(), 
        };
        
        db_crud::add_price(&pool, price, 3).await.expect("Failed to add price");
        db_crud::add_trade(&pool, trade).await.expect("Failed to add trade");

        // get last trade and print it
        let last_trade = db_crud::get_last_trade(&pool).await.expect("Failed to load last trade");

        // get last 3 prices
        let last_3_prices = db_crud::get_last_n_prices(&pool, 3).await.expect("Failed to load prices");

        println!("Trade: {:?}", last_trade);
        println!("Prices: {:?}", last_3_prices);
        
    }
}