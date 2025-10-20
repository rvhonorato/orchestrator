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

            break; // We only expect one file
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::loader::Config;
    use crate::routes::router::AppState;
    use axum::body::to_bytes;
    use axum::body::Body;
    use axum::{routing::post, Router};
    use http::{header, Request, StatusCode};
    use sqlx::SqlitePool;
    use std::path::PathBuf;
    use tempfile::tempdir;
    use tower::ServiceExt; // for `oneshot`
    use uuid::Uuid;

    // Helper function to initialize the database schema
    pub async fn init_db(pool: &SqlitePool) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
        CREATE TABLE IF NOT EXISTS payloads (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            status TEXT NOT NULL,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )
    "#,
        )
        .execute(pool)
        .await?;
        Ok(())
    }

    // Helper functions to create multipart form data
    fn form_file(
        boundary: &str,
        name: &str,
        filename: &str,
        content_type: &str,
        content: &[u8],
    ) -> Vec<u8> {
        let mut part = format!(
            "--{boundary}\r\n\
                Content-Disposition: form-data; name=\"{name}\"; filename=\"{filename}\"\r\n\
                Content-Type: {content_type}\r\n\r\n"
        )
        .into_bytes();
        part.extend_from_slice(content);
        part.extend_from_slice(b"\r\n");
        part
    }

    async fn setup_test_router(endpoint: &str) -> Router {
        // Setup the route
        let data_dir = tempdir().unwrap();
        let mut config = Config::new().unwrap();
        config.data_path = data_dir.path().to_str().unwrap().to_string();
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        init_db(&pool).await.unwrap(); // Initialize the database schema
        let state = AppState { pool, config };

        Router::new()
            .route(endpoint, post(submit))
            .with_state(state)
    }

    #[tokio::test]
    async fn test_submit() {
        let endpoint = "/submit";
        let test_app = setup_test_router(endpoint).await;

        // Create a multipart/form-data request
        let boundary = format!("----Boundary{}", Uuid::new_v4());
        let mut body = Vec::new();
        body.extend(form_file(
            &boundary,
            "file",
            "test.dat",
            "application/octet-stream",
            b"\x00\x01\x02\x03",
        ));
        body.extend(format!("--{boundary}--\r\n").as_bytes());

        // Create the request
        let req = Request::builder()
            .method("POST")
            .uri(endpoint)
            .header(
                header::CONTENT_TYPE,
                format!("multipart/form-data; boundary={boundary}"),
            )
            .body(Body::from(body))
            .unwrap();

        // Make the request
        let response = test_app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["id"], 1);
        assert_eq!(json["status"], String::from("Prepared"));

        // Check if the file was saved correctly
        let expected_loc = json["loc"].as_str().unwrap();
        let expected_file = PathBuf::from(expected_loc).join("test.dat");
        assert!(expected_file.exists());
    }
}
