use crate::utils;
use axum::http::StatusCode;
use axum::{body::Bytes, BoxError};
use futures::Stream;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

pub const UPLOADS_DIRECTORY: &str = "uploads";

#[derive(Deserialize)]
pub struct UploadPayload {
    pub user_id: i32,
    // service: String,
    // access_level: u8,
}

#[derive(serde::Serialize)]
pub struct Job {
    id: i32,
    pub user_id: i32,
    status: String, // placeholder
    loc: PathBuf,   // Path in the filesystem of where this data is saved
}

pub async fn create_jobs_table(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS jobs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id INTEGER NOT NULL,
            status TEXT NOT NULL,
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
            status: "".to_string(),
            loc,
        }
    }

    pub async fn save_to_disk<S, E>(
        &self,
        stream: S,
        filename: String,
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
        let result = sqlx::query("INSERT INTO jobs (user_id, status) VALUES (?, ?)")
            .bind(self.user_id)
            .bind(&self.status)
            .execute(pool)
            .await?;

        let job_id = result.last_insert_rowid();
        self.id = job_id as i32;

        Ok(())
    }

    pub fn set_user_id(&mut self, user_id: i32) {
        self.user_id = user_id;
    }
}

#[derive(Serialize)]
pub struct Ping {
    pub message: String,
}
