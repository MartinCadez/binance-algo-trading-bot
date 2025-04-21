pub mod database_logic;

use dotenv::dotenv;
use std::env;

use database_logic::db_connect;
use database_logic::db_tables;
use database_logic::db_objects::{Column, Tables};
use database_logic::db_crud;

#[tokio::main]
async fn main() {
    dotenv().ok(); // Load .env if present

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    
    // create pool
    let pool = db_connect::connect_to_database(&database_url).await.expect("Failed connecting");
    
    // create table
    let table_name = Tables::Test;
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

    let trades_table_name = Tables::Test;

    db_tables::drop_table(&pool, "Test_table_2").await.expect("Failed to drop table");
    db_tables::create_custom_table(&pool, table_name, columns_for_trades).await.expect("Failed to create custom table");

    // create 2 rows, get second row by id and delete it
    db_crud::create_trade(&pool, "Alice", 2.5).await.expect("");
    let new_trade_2 = db_crud::create_trade(&pool, "Branko", 2.6).await.expect("");
    let user = db_crud::get_row_by_id(&pool, &trades_table_name, new_trade_2.id).await.expect("");
    println!("Trade 2: {:?}", user);
    db_crud::delete_row_by_id(&pool, &trades_table_name, 2).await.expect("");
    
    // drop that table
    db_tables::drop_table(&pool, "test_table_2").await.expect("Failed to drop table");
}
