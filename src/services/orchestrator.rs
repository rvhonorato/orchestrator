use crate::models::job_dao::Job;
use crate::services::jobd::Jobd;
use anyhow::Result;
use axum::http::StatusCode;

#[derive(Debug, thiserror::Error)]
pub enum UploadError {
    #[error("Invalid path")]
    InvalidPath,
    #[error("Failed to encode file: {0}")]
    EncodingFailed(#[from] std::io::Error),
    #[error("Request failed: {0}")]
    RequestFailed(#[from] reqwest::Error),
    #[error("Failed to read response: {0}")]
    ResponseReadFailed(reqwest::Error),
    #[error("Failed to deserialize response: {0}")]
    DeserializationFailed(#[from] serde_json::Error),
    #[error("Unexpected status code: {0}")]
    UnexpectedStatus(StatusCode),
}
#[derive(Debug, thiserror::Error)]
pub enum DownloadError {
    #[error("Not found")]
    NotFound,
    #[error("Invalid path")]
    InvalidPath,
    #[error("Request failed: {0}")]
    RequestFailed(#[from] reqwest::Error),
    #[error("Not finished")]
    NotReady,
    #[error("Failed to deserialize response: {0}")]
    DeserializationFailed(#[from] serde_json::Error),
    #[error("Failed to read response: {0}")]
    ResponseReadFailed(reqwest::Error),
    #[error("Unexpected status code: {0}")]
    UnexpectedStatus(StatusCode),
}

pub async fn send(job: &Job, dest: Destinations) -> Result<String, UploadError> {
    let target = match dest {
        Destinations::Jobd => Jobd,
    };

    target.upload(job).await
}

pub async fn retrieve(job: &Job, dest: Destinations) -> Result<(), DownloadError> {
    if job.id == 0 {
        Err(DownloadError::NotFound)
    } else {
        let target = match dest {
            Destinations::Jobd => Jobd,
        };

        target.download(job).await
    }
}

// pub async fn status() {}

//==================================================================
// Here !list all possible destinations
pub enum Destinations {
    Jobd,
    // Slurml,
    // Dirac,
    // Cloud,
    // etc
}

// These are traits that all Desinations need to have
pub trait Endpoint {
    async fn upload(&self, j: &Job) -> Result<String, UploadError>;
    // async fn status(&self, j: &Job) -> Result<reqwest::Response, reqwest::Error>;
    async fn download(&self, j: &Job) -> Result<(), DownloadError>;
}
