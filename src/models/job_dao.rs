use crate::config::constants;
use crate::models::status_dto::Status;
use crate::utils::io::stream_to_file;
use axum::http::StatusCode;
use axum::{body::Bytes, BoxError};
use futures::Stream;
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

#[derive(serde::Serialize)]
pub struct Job {
    pub id: i32,
    pub user_id: i32,
    pub status: Status,
    pub loc: PathBuf,
    pub dest_id: String,
}

impl Job {
    pub fn new() -> Job {
        let loc =
            std::path::Path::new(constants::UPLOADS_DIRECTORY).join(Uuid::new_v4().to_string());
        match fs::create_dir(&loc) {
            Ok(_) => (),
            Err(e) => println!("could not create directory {}", e),
        }
        Job {
            id: 0,
            user_id: 0,
            status: Status::Pending,
            loc,
            dest_id: String::new(),
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
        stream_to_file(full_path, stream).await?;
        Ok(())
    }
}
