// TODO create table, drop table, check if exists, clear table, 
use::sqlx::{PgPool, Error};
use super::db_objects::{Column, Tables};

#[allow(dead_code)]
pub async fn create_custom_table(pool: &PgPool, table_name: Tables, columns: Vec<Column>) -> Result<(), Error> {
    // Start constructing the SQL query
    let mut query = format!("CREATE TABLE IF NOT EXISTS {} (", table_name.as_str());

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
                table_name.as_str(),
                column_definitions.len()
            );
            Ok(())
        }
        Err(e) => {
            eprintln!("Failed to create table '{}': {}", table_name.as_str(), e);
            // You can also log to a file or return a custom error here if needed
            Err(e)
        }
    }
}

#[allow(dead_code)]
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

#[allow(dead_code)]
pub async fn create_trades_table(pool: &PgPool) -> Result<(), Error> {
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
            },
            Column {
                name: "price".to_string(),
                col_type: "Double precision".to_string(),
                constraints: None,
            },
            Column {
                name: "amount".to_string(),
                col_type: "Double precision".to_string(),
                constraints: None,
            }, 
            Column {
                name: "timestamp".to_string(),
                col_type: "INTEGER".to_string(),
                constraints: None,
            },
            Column {
                name: "State".to_string(),
                col_type: "TEXT".to_string(),
                constraints: None,
            }
        ];
    
    let table_name = Tables::Trades;
    create_custom_table(&pool, table_name, columns_for_trades).await.expect("Failed to create custom table");
    Ok(())
}

#[allow(dead_code)]
pub async fn create_prices_table(pool: &PgPool) -> Result<(), Error> {
    let columns_for_prices = vec![
        Column {
            name: "id".to_string(),
            col_type: "SERIAL".to_string(),
            constraints: Some("PRIMARY KEY".to_string()),
        },
        Column {
            name: "coin".to_string(),
            col_type: "TEXT".to_string(),
            constraints: None,
        },
        Column {
            name: "open".to_string(),
            col_type: "DOUBLE PRECISION".to_string(),
            constraints: None,
        },
        Column {
            name: "high".to_string(),
            col_type: "DOUBLE PRECISION".to_string(),
            constraints: None,
        },
        Column {
            name: "low".to_string(),
            col_type: "DOUBLE PRECISION".to_string(),
            constraints: None,
        },
        Column {
            name: "close".to_string(),
            col_type: "DOUBLE PRECISION".to_string(),
            constraints: None,
        },
        Column {
            name: "volume".to_string(),
            col_type: "DOUBLE PRECISION".to_string(),
            constraints: None,
        },
        Column {
            name: "timestamp".to_string(),
            col_type: "INTEGER".to_string(),
            constraints: None,
        }
    ];

    let table_name = Tables::Prices;
    create_custom_table(pool, table_name, columns_for_prices)
        .await
        .expect("Failed to create prices table");

    Ok(())
}