use crate::config::constants;
use crate::models::job_dao::Job;
use crate::services::orchestrator::Endpoint;
use crate::services::orchestrator::UploadError;
use crate::utils::io::stream_file_to_base64;
use axum::http::StatusCode;
use serde_json::json;
use tracing::debug;
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
    // #[serde(rename = "Output")]
    // output: String,
    // #[serde(rename = "Path")]
    // path: String,
    // #[serde(rename = "SlurmID")]
    // slurm_id: i32,
    // #[serde(rename = "Slurml")]
    // slurml: bool,
    // #[serde(rename = "Status")]
    // status: String,
}

impl Endpoint for Jobd {
    async fn upload(&self, j: &Job) -> Result<String, UploadError> {
        let path = j.loc.join("payload.zip");
        let input_as_base64 =
            stream_file_to_base64(path.to_str().ok_or(UploadError::InvalidPath)?)?;

        let data = json!({
            "id": j.id.to_string(),
            "input": input_as_base64,
            "slurml": false
        });

        let client = reqwest::Client::new();
        let response = client
            .post(constants::JOBD_UPLOAD_ENDPOINT)
            .json(&data)
            .send()
            .await
            .map_err(UploadError::RequestFailed)?;

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

    // async fn download(&self, j: &Job) -> Result<reqwest::Response, reqwest::Error> {
    //     let url = constants::JOBD_DOWNLOAD_ENDPOINT.to_string() + &j.id.to_string();
    //
    //     let client = reqwest::Client::new();
    //     let response = client.get(url).send().await?;
    //
    //     Ok(response)
    // }
}
