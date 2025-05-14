pub mod database_logic;
pub mod utils;

use dotenv::dotenv;
use std::env;

use database_logic::db_connect;
use database_logic::db_tables;
use database_logic::db_objects::{Column, Tables};
use database_logic::db_crud;

#[tokio::main]
async fn main() {
    
}
