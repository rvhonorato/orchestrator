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

#[derive(serde::Deserialize, serde::Serialize, Debug)]
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

fn construct_download_url(base_url: &str, dest_id: &str) -> String {
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
        let download_url = construct_download_url(url, &j.dest_id);
        let response_body = fetch_download_response(&download_url).await?;
        let output_base64 = parse_download_response(&response_body)?;
        save_output_file(&output_base64, &j.loc)
    }
}

#[cfg(test)]
mod test {

    use std::fs::File;
    use std::io::Write;
    use std::path::PathBuf;

    use tempfile::TempDir;

    use super::*;

    #[tokio::test]
    async fn test_prepare_upload_data() {
        // Test ok
        {
            let mut job = Job::new();
            let tempdir = TempDir::new().unwrap();
            let file_path = tempdir.path().join("payload.zip");
            let mut tempfile = File::create(file_path).unwrap();
            writeln!(tempfile, "test").unwrap();

            job.loc = tempdir.into_path();

            let result = prepare_upload_data(&job).await;

            assert!(result.is_ok());
        }
        // Test err
        {
            let mut job = Job::new();

            job.loc = PathBuf::new();

            let result = prepare_upload_data(&job).await;

            assert!(result.is_err());
        }
    }

    #[tokio::test]
    async fn test_sent_post_request() {
        // Test ok
        {
            let mut server = mockito::Server::new_async().await;

            let test_data = json!({
                "key": "value"
            });

            let jobd_response = JobdResponse {
                id: "".to_string(),
                output: "".to_string(),
            };

            let mock = server
                .mock("POST", "/")
                .with_status(201)
                .with_header("content-type", "application/json")
                .with_body(serde_json::to_string(&jobd_response).unwrap())
                .create_async()
                .await;

            let result = send_post_request(&server.url(), test_data).await;

            mock.assert_async().await;

            assert!(result.is_ok());
        }
        // Test err
        {
            let mut server = mockito::Server::new_async().await;

            let test_data = json!({
                "key": "value"
            });

            // Mock the server returning anything else than `201`
            let mock = server
                .mock("POST", "/")
                .with_status(404)
                .with_header("content-type", "application/json")
                .create_async()
                .await;

            let result = send_post_request(&server.url(), test_data).await;

            mock.assert_async().await;

            assert!(result.is_err());
        }
    }

    #[tokio::test]
    async fn test_handle_upload_response() {
        // Test successfull
        {
            let jobd_response = JobdResponse {
                id: "test-id".to_string(),
                output: "test-output".to_string(),
            };
            let response_body = serde_json::to_string(&jobd_response).unwrap();

            let http_response = http::Response::builder()
                .status(StatusCode::CREATED)
                .body(response_body)
                .unwrap();
            let response = reqwest::Response::from(http_response);

            let result = handle_upload_response(response).await;
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), "test-id");
        }
        // Test unexpected status code
        {
            let http_response = http::Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body("".to_string())
                .unwrap();
            let response = reqwest::Response::from(http_response);

            let result = handle_upload_response(response).await;
            assert!(matches!(result, Err(UploadError::UnexpectedStatus(_))));
        }
        // Test malformed JSON response
        {
            let http_response = http::Response::builder()
                .status(StatusCode::CREATED)
                .body("invalid json".to_string())
                .unwrap();
            let response = reqwest::Response::from(http_response);

            let result = handle_upload_response(response).await;
            assert!(matches!(result, Err(UploadError::DeserializationFailed(_))));
        }
    }

    #[test]
    fn test_construct_download_url() {
        let result = construct_download_url("http://localhost", "endpoint");
        assert_eq!(result, "http://localhost/endpoint")
    }

    #[tokio::test]
    async fn test_fetch_download_response() {
        // Test ok 200
        {
            let mut server = mockito::Server::new_async().await;

            let mock = server
                .mock("GET", "/")
                .with_status(200)
                .with_header("content-type", "application/json")
                // .with_body(serde_json::to_string(&jobd_response).unwrap())
                .create_async()
                .await;

            let result = fetch_download_response(&server.url()).await;

            mock.assert_async().await;

            assert!(result.is_ok());
        }
        // Test 202
        {
            let mut server = mockito::Server::new_async().await;

            let mock = server
                .mock("GET", "/")
                .with_status(202)
                .with_header("content-type", "application/json")
                .create_async()
                .await;

            let result = fetch_download_response(&server.url()).await;

            mock.assert_async().await;

            assert!(result.is_err());
        }
        // Test any
        {
            let mut server = mockito::Server::new_async().await;

            let mock = server
                .mock("GET", "/")
                .with_status(503)
                .with_header("content-type", "application/json")
                .create_async()
                .await;

            let result = fetch_download_response(&server.url()).await;

            mock.assert_async().await;

            assert!(result.is_err());
        }
    }

    #[tokio::test]
    async fn test_handle_download_response() {
        // Test successfull 200
        {
            let jobd_response = JobdResponse {
                id: "test-id".to_string(),
                output: "test-output".to_string(),
            };
            let response_body = serde_json::to_string(&jobd_response).unwrap();

            let http_response = http::Response::builder()
                .status(StatusCode::OK)
                .body(response_body)
                .unwrap();
            let response = reqwest::Response::from(http_response);

            let result = handle_download_response(response).await;
            assert!(result.is_ok());
        }
        // Test err 202
        {
            let jobd_response = JobdResponse {
                id: "test-id".to_string(),
                output: "test-output".to_string(),
            };
            let response_body = serde_json::to_string(&jobd_response).unwrap();

            let http_response = http::Response::builder()
                .status(StatusCode::ACCEPTED)
                .body(response_body)
                .unwrap();
            let response = reqwest::Response::from(http_response);

            let result = handle_download_response(response).await;
            assert!(result.is_err());
        }
        // Test err any
        {
            let jobd_response = JobdResponse {
                id: "test-id".to_string(),
                output: "test-output".to_string(),
            };
            let response_body = serde_json::to_string(&jobd_response).unwrap();

            let http_response = http::Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(response_body)
                .unwrap();
            let response = reqwest::Response::from(http_response);

            let result = handle_download_response(response).await;
            assert!(result.is_err());
        }
    }

    #[test]
    fn test_parse_download_response() {
        // Test successful case
        let jobd_response = JobdResponse {
            id: "test-id".to_string(),
            output: "test-output".to_string(),
        };
        let response_body = serde_json::to_string(&jobd_response).unwrap();
        let result = parse_download_response(&response_body);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "test-output");

        // Test malformed JSON response
        let result = parse_download_response("invalid json");
        assert!(matches!(
            result,
            Err(DownloadError::DeserializationFailed(_))
        ));
    }

    #[test]
    fn test_save_output_file() {
        // Create a temporary directory that will be automatically cleaned up
        let temp_dir = TempDir::new().unwrap();
        let job_location = temp_dir.path();

        // Test successful case with valid base64
        // This is a tiny valid ZIP file encoded in base64
        let valid_base64 = "UEsFBgAAAAAAAAAAAAAAAAAAAAAAAA==";
        let result = save_output_file(valid_base64, job_location);
        assert!(result.is_ok());

        // Verify the file was created
        let output_path = job_location.join("output.zip");
        assert!(output_path.exists());

        // Test with invalid base64
        let invalid_base64 = "invalid-base64!@#$";
        let result = save_output_file(invalid_base64, job_location);
        assert!(matches!(result, Err(DownloadError::InvalidPath)));

        // Test with invalid path
        let invalid_path = std::path::Path::new("/nonexistent/directory");
        let result = save_output_file(valid_base64, invalid_path);
        assert!(matches!(result, Err(DownloadError::InvalidPath)));
    }

    #[tokio::test]
    async fn test_jobd_upload() {
        // NOTE: This is an integration test
        let mut server = mockito::Server::new_async().await;

        // Create test job
        let mut job = Job::new();
        let tempdir = TempDir::new().unwrap();
        let file_path = tempdir.path().join("payload.zip");
        let mut tempfile = File::create(file_path).unwrap();
        writeln!(tempfile, "test").unwrap();
        job.loc = tempdir.into_path();

        // Create expected response
        let jobd_response = JobdResponse {
            id: "test-id".to_string(),
            output: "test-output".to_string(),
        };

        // Set up mock server
        let mock = server
            .mock("POST", "/")
            .with_status(201)
            .with_header("content-type", "application/json")
            .with_body(serde_json::to_string(&jobd_response).unwrap())
            .create_async()
            .await;

        // Test the upload
        let jobd = Jobd {};
        let result = jobd.upload(&job, &server.url()).await;

        mock.assert_async().await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "test-id");
    }

    #[tokio::test]
    async fn test_jobd_download() {
        let mut server = mockito::Server::new_async().await;
        let temp_dir = tempfile::TempDir::new().unwrap();

        // Create test job
        let mut job = Job::new();
        job.loc = temp_dir.into_path();

        let dest_id = "test-123".to_string();
        job.dest_id = dest_id.clone();

        // Create test response
        let jobd_response = JobdResponse {
            id: dest_id.clone(),
            output: "UEsFBgAAAAAAAAAAAAAAAAAAAAAAAA==".to_string(), // valid base64 zip
        };

        // Set up mock server
        let mock = server
            .mock("GET", format!("/{}", dest_id).as_str())
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(serde_json::to_string(&jobd_response).unwrap())
            .create_async()
            .await;

        // Test the download
        let jobd = Jobd {};
        let result = jobd.download(&job, &server.url()).await;

        mock.assert_async().await;
        assert!(result.is_ok());

        // Verify file was created
        let output_path = job.loc.join("output.zip");
        assert!(output_path.exists());
    }
}
