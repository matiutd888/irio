use anyhow::Result;
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use std::{env, sync::Arc};

pub async fn get_postgres_connection() -> Result<Arc<Pool<Postgres>>> {
    let hostname = env::var("POSTGRES_HOSTNAME").unwrap_or_else(|_| "127.0.0.1".to_string());
    let user = env::var("POSTGRES_USER").unwrap_or_else(|_| "postgres".to_string());
    let port = env::var("POSTGRES_PORT").unwrap_or_else(|_| "5432".to_string());
    let password = env::var("POSTGRES_PASSWORD").unwrap_or_else(|_| "postgres".to_string());
    let postgres_db = env::var("POSTGRES_DB").unwrap_or_else(|_| "postgres_db".to_string());
    let database_url = format!(
        "postgresql://{}:{}@{}:{}/{}",
        user, password, hostname, port, postgres_db
    );

    // Establish a connection pool
    let pool = PgPoolOptions::new()
        .max_connections(10) // Set the maximum number of connections in the pool
        .connect(&database_url)
        .await?;
    Ok(Arc::new(pool))
}
