use sqlx::PgPool;

pub async fn create_db_connection(database_url: &String) -> Option<PgPool> {
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

    #[tokio::test]
    async fn test_connection() {
        dotenv().ok(); // load .env if present

        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env");

        let pool = create_db_connection(&database_url)
            .await
            .expect("Failed to connect to database");

        println!("Successfully created PgPool: {:?}", pool);
    }
}
