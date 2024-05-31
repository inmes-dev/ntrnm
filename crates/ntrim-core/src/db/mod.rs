pub mod simple_record;

use std::sync::OnceLock;
use sqlx::{Acquire, PgPool};
use sqlx::postgres::PgPoolOptions;
use ntrim_tools::tokiort::global_tokio_runtime;
pub use crate::db::simple_record::SimpleMessageRecord;

pub static PG_POOL: OnceLock<PgPool> = OnceLock::new();

pub async fn initialize_pool(addr: &str) {
    let pool = PgPoolOptions::new()
        .max_connections(option_env!("SQL_MAX_CONNECTIONS").map_or(5, |v| v.parse().unwrap()))
        .connect(addr)
        .await
        .expect("Failed to create PgPool");
    PG_POOL.set(pool).expect("Failed to set PgPool");
}

async fn check_database_connection() -> Result<(), anyhow::Error> {
    let pool = PG_POOL.get().unwrap();
    sqlx::query("SELECT 1").execute(pool).await?;
    Ok(())
}

pub async fn ensure_table_exists() -> Result<(), anyhow::Error> {
    check_database_connection().await?;
    let pool = PG_POOL.get().unwrap();
    let result = tokio::try_join!(
        SimpleMessageRecord::create_table(pool),

    )?;

    Ok(())
}