use crate::models::job_dto::create_jobs_table;
use sqlx::{Pool, Sqlite, SqlitePool};

pub async fn init_db() -> Pool<Sqlite> {
    let pool = SqlitePool::connect("sqlite::memory:")
        .await
        .unwrap_or_else(|e| panic!("Database connection failed: {}", e));

    create_jobs_table(&pool)
        .await
        .expect("failed to create the jobs table");

    pool
}
