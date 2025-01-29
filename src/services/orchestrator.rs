use crate::config::loader::Config;
use crate::models::job_dao::Job;
use crate::services::jobd::Jobd;
use anyhow::Result;
use axum::http::StatusCode;
use tracing::info;

#[derive(Debug, thiserror::Error)]
pub enum UploadError {
    #[error("Invalid service")]
    InvalidService,
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
    #[error("Invalid service")]
    InvalidService,
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

pub async fn send(job: &Job, config: &Config) -> Result<String, UploadError> {
    // TODO: One service may have many destinations, figure that out here
    let dest = Destinations::Jobd;
    let target = match dest {
        Destinations::Jobd => Jobd,
    };

    info!("{:?}", job);

    match config.get_upload_url(&job.service) {
        Some(url) => Ok(target.upload(job, url).await?),
        None => Err(UploadError::InvalidService),
    }
}

pub async fn retrieve(job: &Job, config: &Config) -> Result<(), DownloadError> {
    if job.id == 0 {
        Err(DownloadError::NotFound)
    } else {
        // let target = match dest {
        //     Destinations::Jobd => Jobd,
        // };
        // TODO: One service may have many destinations, figure that out here
        let dest = Destinations::Jobd;
        let target = match dest {
            Destinations::Jobd => Jobd,
        };

        // target.download(job).await
        match config.get_download_url(&job.service) {
            Some(url) => Ok(target.download(job, url).await?),
            None => Err(DownloadError::InvalidService),
        }
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
    async fn upload(&self, j: &Job, url: &str) -> Result<String, UploadError>;
    // async fn status(&self, j: &Job) -> Result<reqwest::Response, reqwest::Error>;
    async fn download(&self, j: &Job, url: &str) -> Result<(), DownloadError>;
}
