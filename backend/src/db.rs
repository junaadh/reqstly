use sqlx::{PgPool, Pool, Postgres};
pub type DbPool = Pool<Postgres>;

pub async fn create_pool(database_url: &str) -> Result<DbPool, sqlx::Error> {
    let pool = PgPool::connect(database_url).await?;

    // Test the connection
    sqlx::query("SELECT 1")
        .fetch_one(&pool)
        .await?;

    Ok(pool)
}

pub async fn run_migrations(pool: &DbPool) -> Result<(), sqlx::migrate::MigrateError> {
    sqlx::migrate!("./migrations").run(pool).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_pool_creation() {
        // This test requires a database to be running
        // Skip in CI if no database available
        let url = std::env::var("DATABASE_URL");
        if url.is_err() {
            return;
        }

        let pool = create_pool(&url.unwrap()).await;
        assert!(pool.is_ok());
    }
}
