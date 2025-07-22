use crate::models::job_dao::Job;
use crate::models::status_dto::Status;
use crate::models::uploadpayload_dto::UploadPayload;
use crate::routes::router::AppState;
use axum::{
    extract::{Json, Multipart, Path, State},
    http::StatusCode,
};
use std::collections::HashMap;
use utoipa;
use utoipa::ToSchema;

#[utoipa::path(
    get,
    path = "/download/{id}",
    params(
        ("id" = i32, Path, description = "Job identifier")
    ),
    responses(
        (status = 200, description = "File downloaded successfully", body = Vec<u8>),
        (status = 202, description = "Job not ready"),
        (status = 204, description = "Job failed or cleaned"),
        (status = 404, description = "Job not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "files"
)]
pub async fn download(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Vec<u8>, StatusCode> {
    let mut job = Job::new(&state.config.data_path);

    job.retrieve_id(id, &state.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        })?;

    match job.status {
        Status::Completed => Ok(job.download()),
        Status::Failed => Err(StatusCode::NO_CONTENT),
        Status::Cleaned => Err(StatusCode::NO_CONTENT),
        // TODO: Handle other status here
        _ => Err(StatusCode::ACCEPTED),
    }
}

#[derive(ToSchema)]
#[allow(dead_code)]
struct MultipartUpload {
    #[schema(format = "binary", value_type = String)]
    file: Vec<u8>,
    #[schema(example = "{\"user_id\": 2, \"service\": \"generic\"}")]
    data: String,
}
#[utoipa::path(
    post,
    path = "/upload",
    request_body(
        content_type = "multipart/form-data",
        content = MultipartUpload,
        description = "Upload file and metadata"
    ),
    responses(
        (status = 200, description = "File uploaded successfully", body = Job),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error"),
        (status = 503, description = "Service unavailable")
    ),
    tag = "files"
)]
pub async fn upload(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<Json<Job>, (StatusCode, String)> {
    let mut user_data = None;

    let mut text_fields: HashMap<String, String> = HashMap::new();
    let mut file_fields: HashMap<String, String> = HashMap::new(); // field_name -> filename
    let mut job = Job::new(&state.config.data_path);

    // Collect all fields
    while let Ok(Some(field)) = multipart.next_field().await {
        if let Some(field_name) = field.name() {
            let field_name = field_name.to_string();

            if field.file_name().is_some() {
                // Handle file
                let original_filename = field.file_name().unwrap().to_string();
                let safe_filename = format!("{}_{}", field_name, original_filename);

                job.save_to_disk(field, &safe_filename).await?;
                file_fields.insert(field_name, safe_filename);
            } else {
                // Handle text
                let text_data = field
                    .text()
                    .await
                    .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
                text_fields.insert(field_name, text_data);
            }
        }
    }

    // TODO: Process
    todo!()

    // TODO: Store metadata?
    //

    job.set_user_id(user_data.user_id);
    job.set_service(user_data.service);

    // Add it to the database and handle potential errors
    job.add_to_db(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let _ = job.update_status(Status::Queued, &state.pool).await;

    Ok(Json(job))
}
