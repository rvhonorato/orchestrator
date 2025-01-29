use crate::models::job_dto::create_jobs_table;
use sqlx::{Pool, Sqlite, SqlitePool};
use std::env;
use tracing::info;

pub async fn init_db() -> Pool<Sqlite> {
    let wd_path = env::var("ORCHESTRATOR_DATA_PATH").expect("ORCHESTRATOR_DATA_PATH not defined");
    let connection_string = format!("sqlite://{}/db.sqlite?mode=rwc", wd_path);
    info!("Using database: {}", connection_string);
    let pool = SqlitePool::connect(&connection_string)
        .await
        .unwrap_or_else(|e| panic!("Database connection failed: {}", e));

    create_jobs_table(&pool)
        .await
        .expect("failed to create the jobs table");

    pool
}
