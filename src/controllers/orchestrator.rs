use crate::models::job_dao::Job;
use crate::models::status_dto::Status;
use crate::models::uploadpayload_dto::UploadPayload;
use axum::{
    extract::{Json, Multipart, Path, State},
    http::StatusCode,
};
use sqlx::SqlitePool;
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
        (status = 500, description = "Internal server error")
    ),
    tag = "files"
)]
pub async fn download(
    State(pool): State<SqlitePool>,
    Path(id): Path<i32>,
) -> Result<Vec<u8>, StatusCode> {
    let mut job = Job::new();

    job.retrieve_id(id, &pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match job.status {
        Status::Completed => Ok(job.download()),
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
        (status = 500, description = "Internal server error")
    ),
    tag = "files"
)]
pub async fn upload(
    State(pool): State<SqlitePool>,
    mut multipart: Multipart,
) -> Result<Json<Job>, (StatusCode, String)> {
    let mut user_data = None;
    // Create an empty job
    let mut job = Job::new();

    while let Ok(Some(field)) = multipart.next_field().await {
        if let Some(field_name) = field.name() {
            match field_name {
                "file" => {
                    // Save the file to disk
                    // let filename = field.file_name().unwrap().to_string();

                    let filename = "payload.zip".to_string();

                    job.save_to_disk(field, &filename).await?;

                    // // TODO: Make sure the file is a zip file
                    // if !is_zip(&filename) {
                    //     todo!()
                    // }
                }
                "data" => {
                    // Extract relevant fields from the data
                    let data = field
                        .text()
                        .await
                        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;

                    // Map the json to the `UploadPayload` struct
                    user_data = Some(
                        serde_json::from_str::<UploadPayload>(&data)
                            .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?,
                    );
                }
                _ => {}
            }
        }
    }
    let user_data = user_data.ok_or((StatusCode::BAD_REQUEST, "Missing JSON data".to_string()))?;
    job.set_user_id(user_data.user_id);
    job.set_service(user_data.service);

    // Add it to the database and handle potential errors
    job.add_to_db(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let _ = job.update_status(Status::Queued, &pool).await;

    Ok(Json(job))
}
