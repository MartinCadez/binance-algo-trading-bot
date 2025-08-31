/// UNUSED: 
/// tables management utilities are currently not used since, whole db and
/// all db-schemas are created initially with docker, basically there is no 
/// need for dynamic interaction with db, kept here in case for potential need

use::sqlx::{PgPool, Error};
use super::objects::{Column, Tables};

pub async fn create_custom_table(pool: &PgPool, table_name: Tables, columns: Vec<Column>) -> Result<(), Error> {
    // start constructing the SQL query
    let mut query = format!("CREATE TABLE IF NOT EXISTS {} (", table_name.as_str());

    // add columns to the query
    let column_definitions: Vec<String> = columns.into_iter().map(|col| {
        let mut definition = format!("{} {}", col.name, col.col_type);
        if let Some(constraint) = col.constraints {
            definition.push_str(&format!(" {}", constraint));
        }
        definition
    }).collect();

    // join all columns into a single string and append to query
    query.push_str(&column_definitions.join(", "));
    query.push_str(");");

    // execute the query to create the table
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
            // you can also log to a file or return a custom error here if needed
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