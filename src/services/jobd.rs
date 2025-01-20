use crate::config::constants;
use crate::models::job_dao::Job;
use crate::services::orchestrator::Endpoint;
use crate::utils::io::stream_file_to_base64;
use serde_json::{json, Value};

//-------------------------
// jobd
//-------------------------
pub struct Jobd;

#[derive(serde::Deserialize, Debug)]
struct JobdResponse {
    id: String,
}

impl Endpoint for Jobd {
    async fn upload(&self, j: &Job) -> Result<String, reqwest::Error> {
        let input_as_base64 =
            stream_file_to_base64(j.loc.join("payload.zip").to_str().unwrap()).unwrap();

        let data = json!({
            "id": j.id.to_string(),
            "input": input_as_base64,
            "slurml": false
        });

        let url = constants::JOBD_UPLOAD_ENDPOINT;

        let client = reqwest::Client::new();
        let response = client
            .post(url)
            .json(&data)
            .send()
            .await?
            .json::<JobdResponse>()
            .await?;

        // TODO: The response will contain a jobd specific job_id, parse it and add to the Job
        // let jobd_id = "something";
        // j.update_dest_id(jobd_id);
        //
        //
        // TODO: STOPPED HERE - FIGURE OUT HOW TO GET THE JOB_ID RETURNED FROM JOBD

        Ok(response.id)
    }

    async fn download(&self, j: &Job) -> Result<reqwest::Response, reqwest::Error> {
        let url = constants::JOBD_DOWNLOAD_ENDPOINT.to_string() + &j.id.to_string();

        let client = reqwest::Client::new();
        let response = client.get(url).send().await?;

        Ok(response)
    }
}
