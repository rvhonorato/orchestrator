use crate::utils;
use axum::http::StatusCode;
use axum::{body::Bytes, BoxError};
use futures::Stream;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::fmt;
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

pub const UPLOADS_DIRECTORY: &str = "uploads";

#[derive(Deserialize)]
pub struct UploadPayload {
    pub user_id: i32,
    // access_level: u8,
}

#[derive(serde::Serialize)]
pub struct Job {
    id: i32,
    pub user_id: i32,
    status: Status,
    pub loc: PathBuf,
}

pub async fn create_jobs_table(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS jobs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id INTEGER NOT NULL,
            status TEXT NOT NULL,
            loc TEXT NOT NULL,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )
    "#,
    )
    .execute(pool)
    .await?;
    Ok(())
}

impl Job {
    pub fn new() -> Job {
        let loc = std::path::Path::new(UPLOADS_DIRECTORY).join(Uuid::new_v4().to_string());
        match fs::create_dir(&loc) {
            Ok(_) => (),
            Err(e) => println!("could not create directory {}", e),
        }
        Job {
            id: 0,
            user_id: 0,
            status: Status::Pending,
            loc,
        }
    }

    pub async fn save_to_disk<S, E>(
        &self,
        stream: S,
        filename: &String,
    ) -> Result<(), (StatusCode, String)>
    where
        S: Stream<Item = Result<Bytes, E>>,
        E: Into<BoxError>,
    {
        let full_path = std::path::Path::join(&self.loc, filename);
        utils::stream_to_file(full_path, stream).await?;
        Ok(())
    }

    pub async fn add_to_db(&mut self, pool: &SqlitePool) -> Result<(), sqlx::Error> {
        let result = sqlx::query("INSERT INTO jobs (user_id, loc, status) VALUES (?, ?, ?)")
            .bind(self.user_id)
            .bind(self.loc.to_str())
            .bind(self.status.to_string())
            .execute(pool)
            .await?;

        let job_id = result.last_insert_rowid();
        self.id = job_id as i32;

        Ok(())
    }

    pub async fn update_status(
        &mut self,
        status: Status,
        pool: &SqlitePool,
    ) -> Result<(), sqlx::Error> {
        let _result = sqlx::query("UPDATE jobs SET status = ? WHERE id = ?")
            .bind(status.to_string())
            .bind(self.id)
            .execute(pool)
            .await?;

        self.status = status;

        Ok(())
    }

    pub fn set_user_id(&mut self, user_id: i32) {
        self.user_id = user_id;
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Status {
    Pending,
    Processing,
    Completed,
    Failed,
    Queued,
    Unknown,
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Status::Pending => write!(f, "pending"),
            Status::Processing => write!(f, "processing"),
            Status::Completed => write!(f, "completed"),
            Status::Failed => write!(f, "failed"),
            Status::Queued => write!(f, "queued"),
            Status::Unknown => write!(f, "unknown"),
        }
    }
}

#[derive(Serialize)]
pub struct Ping {
    pub message: String,
}
