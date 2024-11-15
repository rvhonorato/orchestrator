use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize)]
pub struct UploadPayload {
    pub user_id: i32,
    // service: String,
    // access_level: u8,
}

#[derive(Serialize)]
pub struct Job {
    pub user_id: i32,
    pub job_id: Uuid,
}

#[derive(Serialize)]
pub struct Ping {
    pub message: String,
}
