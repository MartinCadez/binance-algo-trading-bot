// TODO create table, drop table, check if exists, clear table, 
use::sqlx::{PgPool, Error};
use super::db_objects::Column;

pub async fn create_custom_table(pool: &PgPool, table_name: &str, columns: Vec<Column>) -> Result<(), Error> {
    // Start constructing the SQL query
    let mut query = format!("CREATE TABLE IF NOT EXISTS {} (", table_name);

    // Add columns to the query
    let column_definitions: Vec<String> = columns.into_iter().map(|col| {
        let mut definition = format!("{} {}", col.name, col.col_type);
        if let Some(constraint) = col.constraints {
            definition.push_str(&format!(" {}", constraint));
        }
        definition
    }).collect();

    // Join all the columns into a single string and append to the query
    query.push_str(&column_definitions.join(", "));
    query.push_str(");");

    // Execute the query to create the table
    match sqlx::query(&query).execute(pool).await {
        Ok(_) => {
            println!(
                "Table '{}' created successfully with {} columns.",
                table_name,
                column_definitions.len()
            );
            Ok(())
        }
        Err(e) => {
            eprintln!("Failed to create table '{}': {}", table_name, e);
            // You can also log to a file or return a custom error here if needed
            Err(e)
        }
    }
}

pub async fn drop_table(pool: &PgPool, table_name: &str) -> Result<(), Error> {
    let query = format!("DROP TABLE IF EXISTS {};", table_name);

    match sqlx::query(&query).execute(pool).await {
        Ok(_) => {
            println!("Table '{}' dropped successfully.", table_name);
            Ok(())
        }
        Err(e) => {
            eprintln!("Failed to drop table '{}': {}", table_name, e);
            Err(e)
        }
    }
}

pub async fn clear_table(pool: &PgPool, table_name: &str) -> Result<(), Error> {
    let query = format!("DELETE FROM {};", table_name);

    match sqlx::query(&query).execute(pool).await {
        Ok(result) => {
            println!(
                "Cleared table '{}', removed {} rows.",
                table_name,
                result.rows_affected()
            );
            Ok(())
        }
        Err(e) => {
            eprintln!("Failed to clear table '{}': {}", table_name, e);
            Err(e)
        }
    }
}