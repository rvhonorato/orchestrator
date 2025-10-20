use crate::{routes::router::AppState, utils::io::sanitize_filename};

use crate::models::payload_dao::Payload;
use crate::models::status_dto::Status;
use axum::{
    extract::{Json, Multipart, State},
    http::StatusCode,
};

#[utoipa::path(
    post,
    path = "/submit",
    request_body(
        content_type = "multipart/form-data",
    ),
    responses(
        (status = 200, description = "File uploaded successfully", body = Payload),
        // (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error"),
        // (status = 503, description = "Service unavailable")
    ),
    tag = "files"
)]
pub async fn submit(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<Json<Payload>, (StatusCode, String)> {
    let mut payload = Payload::new();

    // Parse the multipart form data
    while let Some(field) = multipart.next_field().await.unwrap() {
        if let Some(filename) = field.file_name() {
            let clean_filename = sanitize_filename(filename);
            payload.set_filename(clean_filename);

            let data = field.bytes().await.unwrap();
            payload.set_input(data.to_vec());
        }
    }
    tracing::info!("Received payload submission");

    payload
        .add_to_db(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    payload.prepare(&state.config.data_path).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to prepare payload: {e}"),
        )
    })?;

    payload
        .update_status(Status::Prepared, &state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(payload))
}

pub async fn retrieve() {}
