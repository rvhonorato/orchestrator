use crate::config::constants;
use crate::models::job_dao::Job;
use crate::services::orchestrator::Endpoint;
use crate::utils::io::stream_file_to_base64;
use serde_json::json;

//-------------------------
// jobd
//-------------------------
pub struct Jobd;

impl Endpoint for Jobd {
    async fn upload(&self, j: &Job) -> Result<reqwest::Response, reqwest::Error> {
        let input_as_base64 =
            stream_file_to_base64(j.loc.join("payload.zip").to_str().unwrap()).unwrap();

        let data = json!({
            "id": "abc",
            "input": input_as_base64,
            "slurml": false
        });

        let client = reqwest::Client::new();
        let response = client
            .post(constants::JOBD_UPLOAD_ENDPOINT)
            .json(&data)
            .send()
            .await?;

        // TODO: The response will contain a jobd specific job_id, parse it and add to the Job
        // let jobd_id = "something";
        // j.update_dest_id(jobd_id);

        Ok(response)
    }
}
