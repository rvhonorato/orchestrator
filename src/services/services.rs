use crate::models::job_dao::Job;
use crate::utils::utils;
use anyhow::Result;
use serde_json::json;

// FIXME: These need to come from a config!
pub const UPLOADS_DIRECTORY: &str = "uploads";
pub const JOBD_UPLOAD_ENDPOINT: &str = "/upload";

// Public functions
pub async fn send(job: &Job, dest: Destinations) -> Result<bool> {
    let target = match dest {
        Destinations::Jobd => Jobd,
    };

    let _ = target.upload(job).await;

    Ok(true)
}

// pub async fn download() {}
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
trait Endpoint {
    async fn upload(&self, j: &Job) -> Result<reqwest::Response, reqwest::Error>;
    // async fn status(&self, j: &Job) -> Result<reqwest::Response, reqwest::Error>;
    // async fn download(&self, j: &Job) -> Result<reqwest::Response, reqwest::Error>;
}

//-------------------------
// jobd
//-------------------------
struct Jobd;

impl Endpoint for Jobd {
    async fn upload(&self, j: &Job) -> Result<reqwest::Response, reqwest::Error> {
        let input_as_base64 =
            utils::stream_file_to_base64(j.loc.join("payload.zip").to_str().unwrap()).unwrap();

        let data = json!({
            "id": "abc",
            "input": input_as_base64,
            "slurml": false
        });

        let client = reqwest::Client::new();
        let response = client.post(JOBD_UPLOAD_ENDPOINT).json(&data).send().await?;

        // TODO: The response will contain a jobd specific job_id, parse it and add to the Job
        // let jobd_id = "something";
        // j.update_dest_id(jobd_id);

        Ok(response)
    }
}
