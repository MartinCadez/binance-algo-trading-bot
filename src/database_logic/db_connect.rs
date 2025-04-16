// TODO connect to the database with given URL
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
    use crate::db_tables;
    use crate::database_logic::db_objects::Column;

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

        let pool = connect_to_database(&database_url).await.expect("Failed connecting");
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