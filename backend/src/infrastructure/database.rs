use sqlx::{postgres::PgPoolOptions, PgPool};

use crate::shared::errors::AppError;

/// Creates the connection pool. Queries are checked against the live schema at
/// request time; use the `migrate` binary to bring a database up to date first.
pub async fn connect(database_url: &str) -> Result<PgPool, AppError> {
    PgPoolOptions::new()
        .max_connections(16)
        .acquire_timeout(std::time::Duration::from_secs(8))
        .connect(database_url)
        .await
        .map_err(AppError::from)
}
