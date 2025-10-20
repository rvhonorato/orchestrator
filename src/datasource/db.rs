use crate::models::job_dto::create_jobs_table;
use crate::models::payload_dto::create_payload_table;
use sqlx::{Pool, Sqlite, SqlitePool};
use tracing::info;

pub async fn init_db(db_path: &str) -> Pool<Sqlite> {
    let connection_string = format!("sqlite://{db_path}?mode=rwc").to_string();
    info!("Using database: {}", connection_string);
    let pool = SqlitePool::connect(&connection_string)
        .await
        .unwrap_or_else(|e| panic!("Database connection failed: {e}"));

    create_jobs_table(&pool)
        .await
        .expect("failed to create the jobs table");

    pool
}

pub async fn init_payload_db() -> Pool<Sqlite> {
    let connection_string = "sqlite::memory:".to_string();
    info!("Using in-memory database");
    let pool = SqlitePool::connect(&connection_string)
        .await
        .unwrap_or_else(|e| panic!("Database connection failed: {e}"));

    create_payload_table(&pool)
        .await
        .expect("failed to create the payloads table");

    pool
}
