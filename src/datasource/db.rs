use std::fs::File;
use std::{env, path::Path};

use crate::models::job_dto::create_jobs_table;
use sqlx::{Pool, Sqlite, SqlitePool};
use tracing::{info, warn};

pub async fn init_db() -> Pool<Sqlite> {
    // let _db
    let _db = match env::var("ORCHESTRATOR_DB_PATH") {
        Ok(path_str) => {
            let path = Path::new(&path_str);

            if !path.exists() {
                info!(
                    "ORCHESTRATOR_DB_PATH {}, does not exist - creating",
                    path_str
                );
                File::create(path).unwrap();
            } else {
                warn!(
                    "ORCHESTRATOR_DB_PATH {} already exists - using it",
                    path_str
                );
            }
            format!("sqlite:{}", path_str)
        }
        Err(_) => "sqlite::memory:".to_string(),
    };
    let pool = SqlitePool::connect(&_db)
        .await
        .unwrap_or_else(|e| panic!("Database connection failed: {}", e));

    create_jobs_table(&pool)
        .await
        .expect("failed to create the jobs table");

    pool
}
