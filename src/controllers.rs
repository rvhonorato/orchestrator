use crate::models;
use crate::services;
use axum::{
    extract::{Json, Multipart},
    http::StatusCode,
};
use uuid::Uuid;

pub async fn upload(mut multipart: Multipart) -> Result<Json<models::Job>, (StatusCode, String)> {
    let mut user_data = None;
    // TODO: Find a better way of using the job service here
    let job_service = services::JobService;
    while let Ok(Some(field)) = multipart.next_field().await {
        if let Some(field_name) = field.name() {
            // check if this field name is file before proceeding
            if field_name == "file" {
                // TODO: use a proper filename here
                let file_name = Uuid::new_v4().to_string();

                job_service.stream_to_file(&file_name, field).await?;
            } else if field_name == "data" {
                let data = field
                    .text()
                    .await
                    .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
                user_data = Some(
                    serde_json::from_str::<models::UploadPayload>(&data)
                        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?,
                );
            }
        }
    }
    let user_data = user_data.ok_or((StatusCode::BAD_REQUEST, "Missing JSON data".to_string()))?;

    let mut j = job_service.submit().await;
    j.user_id = user_data.user_id;
    Ok(Json(j))
}

pub async fn ping() -> Json<models::Ping> {
    Json(models::Ping {
        message: "pong".to_string(),
    })
}
