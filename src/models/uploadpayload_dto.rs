use serde::Deserialize;
use utoipa::ToSchema;

#[derive(Deserialize, ToSchema)]
pub struct UploadPayload {
    pub user_id: i32,
    pub service: String,
}
