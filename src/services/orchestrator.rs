use crate::models::job_dao::Job;
use crate::services::jobd::Jobd;
use anyhow::Result;

pub async fn send(job: &Job, dest: Destinations) -> Result<String> {
    let target = match dest {
        Destinations::Jobd => Jobd,
    };

    let upload_id = target.upload(job).await?;

    Ok(upload_id)
}

pub async fn download(job: &Job, dest: Destinations) -> Result<bool> {
    let target = match dest {
        Destinations::Jobd => Jobd,
    };

    let _ = target.download(job).await;

    Ok(true)
}
// pub async fn status() {}

//==================================================================
// Here list all possible destinations
pub enum Destinations {
    Jobd,
    // Slurml,
    // Dirac,
    // Cloud,
    // etc
}

// These are traits that all Desinations need to have
pub trait Endpoint {
    async fn upload(&self, j: &Job) -> Result<String, reqwest::Error>;
    // async fn status(&self, j: &Job) -> Result<reqwest::Response, reqwest::Error>;
    async fn download(&self, j: &Job) -> Result<reqwest::Response, reqwest::Error>;
}
