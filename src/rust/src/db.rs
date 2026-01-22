use sqlx::{sqlite::SqlitePoolOptions, Pool, Sqlite};
use std::env;

pub type DbPool = Pool<Sqlite>;

pub async fn init_db() -> Result<DbPool, sqlx::Error> {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    // Create the file if it doesn't exist (handled by sqlite:filename.db?mode=rwc usually, or sqlx create)
    // SQLx requires the file to exist or use sqlx::migrate! with options.
    // For simplicity, we assume the connection string allows creation or we handle it.
    // sqlite://void-eid.db?mode=rwc

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&match database_url.contains("mode=rwc") {
            true => database_url,
            false => format!("{}?mode=rwc", database_url),
        })
        .await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    Ok(pool)
}
