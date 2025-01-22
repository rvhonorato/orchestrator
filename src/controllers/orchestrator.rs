use crate::models::job_dao::Job;
use crate::models::uploadpayload_dto::UploadPayload;
use crate::services::orchestrator;
use axum::{
    extract::{Json, Multipart, State},
    http::StatusCode,
};
use sqlx::SqlitePool;
use tracing::error;

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

    // Add it to the database and handle potential errors
    job.add_to_db(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // -------------------------------------------------------
    // FIXME: This is temporary just to test the service
    let upload_id = match orchestrator::send(&job, orchestrator::Destinations::Jobd).await {
        Ok(id) => id,
        // error coming from the destination
        Err(orchestrator::UploadError::UnexpectedStatus(status)) => {
            return Err((status, "".to_string()))
        }
        // generic error
        Err(e) => {
            error!("{:?}", e);
            return Err((
                reqwest::StatusCode::INTERNAL_SERVER_ERROR,
                "something went wrong while trying to send to the destination".to_string(),
            ));
        }
    };
    // -------------------------------------------------------

    let _ = job.update_dest_id(upload_id, &pool).await;

    Ok(Json(job))
}
