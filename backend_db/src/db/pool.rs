use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::time::Duration;

// Create a connection pool with production-ready settings
pub async fn create_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        // Maximum connections in the pool
        // Adjust based on your database server capacity
        .max_connections(10)
        // Minimum idle connections to maintain
        .min_connections(2)
        // How long to wait for a connection before timing out
        .acquire_timeout(Duration::from_secs(5))
        // Maximum lifetime of a connection
        // Helps prevent issues with stale connections
        .max_lifetime(Duration::from_secs(30 * 60))
        // How long a connection can be idle before being closed
        .idle_timeout(Duration::from_secs(10 * 60))
        .connect(database_url)
        .await
}