use serde::Deserialize;

#[derive(Deserialize)]
pub struct UploadPayload {
    pub user_id: i32,
    pub service: String,
}
