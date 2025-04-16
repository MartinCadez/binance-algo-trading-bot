pub mod database_logic;

use dotenv::dotenv;
use std::env;

use database_logic::db_connect;
use database_logic::db_tables;
use database_logic::db_objects::Column;
use database_logic::db_crud;

#[tokio::main]
async fn main() {
    dotenv().ok(); // Load .env if present

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    
    let pool = db_connect::connect_to_database(&database_url).await.expect("Failed connecting");

    let table_name = "Trades";
    let columns_for_trades= vec![
        Column {
            name: "id".to_string(),
            col_type: "SERIAL".to_string(),
            constraints: Some("PRIMARY KEY".to_string()),
        }, 
        Column {
            name: "coin".to_string(),
            col_type: "TEXT".to_string(),
            constraints: None,
        },Column {
            name: "amount".to_string(),
            col_type: "Double precision".to_string(),
            constraints: None,
        }
    ];
    db_tables::drop_table(&pool, "trades").await.expect("Failed to drop table");
    db_tables::create_custom_table(&pool, &table_name, columns_for_trades).await.expect("Failed to create custom table");
    db_crud::create_trade(&pool, "Alice", 2.5).await.expect("");
    let new_trade_2 = db_crud::create_trade(&pool, "Branko", 2.6).await.expect("");
    let user = db_crud::get_trade_by_id(&pool, new_trade_2.id).await.expect("");
    println!("Trade 2: {:?}", user);
    db_crud::delete_trade(&pool, 2).await.expect("");

    //db_tables::drop_table(&pool, "test_table_2").await.expect("Failed to drop table");
    //db_tables::clear_table(&pool, "users").await.expect("Failed to clear tabel");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_connection() {
        dotenv().ok(); // Load .env if present
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

        let pool = db_connect::connect_to_database(&database_url).await;
        assert!(pool.is_some(), "Did not connect");
    }

    #[tokio::test]
    async fn test_connect_create_and_drop() {
        dotenv().ok(); // Load .env if present
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

        let pool = db_connect::connect_to_database(&database_url).await.expect("Failed connecting");
        let table_name = "test_table_2";
        let columns= vec![
            Column {
                name: "id".to_string(),
                col_type: "TEXT".to_string(),
                constraints: Some("PRIMARY KEY".to_string()),
            }
        ];
        
        db_tables::create_custom_table(&pool, &table_name, columns).await.expect("Failed to create custom table");
        
        db_tables::drop_table(&pool, "test_table_2").await.expect("Failed to drop table");
    }
}
