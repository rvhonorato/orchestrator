use crate::models::job_dao::Job;
use crate::services::orchestrator::Endpoint;
use crate::services::orchestrator::UploadError;
use crate::utils::io::base64_to_file;
use crate::utils::io::stream_file_to_base64;
use axum::http::StatusCode;
use serde_json::json;
use tracing::debug;
use uuid::Uuid;

use super::orchestrator::DownloadError;
//-------------------------
// jobd
//-------------------------
pub struct Jobd;

// #[derive(serde::Deserialize, Debug)]
// struct JobdResponse {
//     id: String,
// }

#[derive(serde::Deserialize, Debug)]
struct JobdResponse {
    #[serde(rename = "ID")]
    id: String,
    // #[serde(rename = "Input")]
    // input: String,
    // #[serde(rename = "LastUpdated")]
    // last_updated: String,
    // #[serde(rename = "Message")]
    // message: String,
    #[serde(rename = "Output")]
    output: String,
    // #[serde(rename = "Path")]
    // path: String,
    // #[serde(rename = "SlurmID")]
    // slurm_id: i32,
    // #[serde(rename = "Slurml")]
    // slurml: bool,
    // #[serde(rename = "Status")]
    // status: String,
}

async fn prepare_upload_data(job: &Job) -> Result<serde_json::Value, UploadError> {
    let path = job.loc.join("payload.zip");
    let input_as_base64 = stream_file_to_base64(path.to_str().ok_or(UploadError::InvalidPath)?)?;

    Ok(json!({
        "id": Uuid::new_v4().to_string(),
        "input": input_as_base64,
        "slurml": false
    }))
}

async fn send_post_request(url: &str, data: serde_json::Value) -> Result<String, UploadError> {
    let client = reqwest::Client::new();
    let response = client
        .post(url)
        .json(&data)
        .send()
        .await
        .map_err(UploadError::RequestFailed)?;

    handle_upload_response(response).await
}

async fn handle_upload_response(response: reqwest::Response) -> Result<String, UploadError> {
    match response.status() {
        StatusCode::CREATED => {
            let body = response
                .text()
                .await
                .map_err(UploadError::ResponseReadFailed)?;

            debug!("{:?}", body);

            let jobd_response = serde_json::from_str::<JobdResponse>(&body)
                .map_err(UploadError::DeserializationFailed)?;

            Ok(jobd_response.id)
        }
        status => Err(UploadError::UnexpectedStatus(status)),
    }
}

async fn construct_download_url(base_url: &str, dest_id: &str) -> String {
    format!("{}/{}", base_url, dest_id)
}

async fn fetch_download_response(url: &str) -> Result<String, DownloadError> {
    let client = reqwest::Client::new();
    debug!("{:?}", url);

    let response = client
        .get(url)
        .send()
        .await
        .map_err(DownloadError::RequestFailed)?;

    handle_download_response(response).await
}

async fn handle_download_response(response: reqwest::Response) -> Result<String, DownloadError> {
    match response.status() {
        StatusCode::OK => response
            .text()
            .await
            .map_err(DownloadError::ResponseReadFailed),
        StatusCode::ACCEPTED => Err(DownloadError::NotReady),
        status => Err(DownloadError::UnexpectedStatus(status)),
    }
}

fn parse_download_response(body: &str) -> Result<String, DownloadError> {
    let jobd_response =
        serde_json::from_str::<JobdResponse>(body).map_err(DownloadError::DeserializationFailed)?;
    Ok(jobd_response.output)
}

fn save_output_file(
    output_base64: &str,
    job_location: &std::path::Path,
) -> Result<(), DownloadError> {
    let output_path = job_location.join("output.zip");
    base64_to_file(output_base64, output_path).map_err(|_| DownloadError::InvalidPath)
}

impl Endpoint for Jobd {
    async fn upload(&self, job: &Job, url: &str) -> Result<String, UploadError> {
        let data = prepare_upload_data(job).await?;
        send_post_request(url, data).await
    }

    async fn download(&self, j: &Job, url: &str) -> Result<(), DownloadError> {
        let download_url = construct_download_url(url, &j.dest_id).await;
        let response_body = fetch_download_response(&download_url).await?;
        let output_base64 = parse_download_response(&response_body)?;
        save_output_file(&output_base64, &j.loc)
    }
}
