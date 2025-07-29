use crate::models::job_dao::Job;
use crate::models::status_dto::Status;
use crate::routes::router::AppState;
use crate::utils::io::{sanitize_filename, save_file};
use axum::{
    extract::{Json, Multipart, Path, State},
    http::StatusCode,
};
use std::collections::HashMap;
use tokio::fs::create_dir_all;
use utoipa;

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

#[utoipa::path(
    post,
    path = "/upload",
    request_body(
        content_type = "multipart/form-data",
        description = "Upload a file and metadata fields as multipart/form-data. \
        The request must include a file field (with any filename and content type), a 'user_id' field (integer), and a 'service' field (string). \
        Additional fields may be included as needed."
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
    // Create a new job with unique ID
    let mut job = Job::new(&state.config.data_path);

    // Create job directory
    create_dir_all(&job.loc).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to create directory: {e}"),
        )
    })?;

    let mut text_fields = HashMap::new();
    let mut file_count = 0;

    // Process each field in the multipart stream
    while let Some(field) = multipart.next_field().await.map_err(|e| {
        tracing::error!("Multipart error: {e}");
        (StatusCode::BAD_REQUEST, format!("Multipart error: {e}"))
    })? {
        let field_name = field.name().unwrap_or("unnamed").to_string();

        if let Some(filename) = field.file_name() {
            file_count += 1;
            let filename = sanitize_filename(filename); // Important for security!
            let file_path = job.loc.join(&filename);

            tracing::info!("Saving file: {} to {}", filename, file_path.display());

            // Create and save the file
            save_file(field, &file_path).await?;
        } else {
            // Handle text field
            let text = field.text().await.map_err(|e| {
                tracing::error!("Error reading text field: {e}");
                (
                    StatusCode::BAD_REQUEST,
                    format!("Error reading text field: {e}"),
                )
            })?;
            text_fields.insert(field_name, text);
        }
    }

    tracing::info!(
        "Upload completed: {} files saved, {} text fields",
        file_count,
        text_fields.len()
    );

    // Now handle special fields
    let user_id = text_fields
        .get("user_id")
        .ok_or((StatusCode::BAD_REQUEST, "Missing user_id".to_string()))?
        .parse::<i32>()
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid user_id".to_string()))?;

    let service = text_fields
        .get("service")
        .ok_or((StatusCode::BAD_REQUEST, "Missing service".to_string()))?
        .to_string();

    // Validate service exists
    if !state.config.services.contains_key(&service) {
        return Err((StatusCode::BAD_REQUEST, "Invalid service".to_string()));
    }

    job.set_user_id(user_id);
    job.set_service(service);

    // Add job to database
    job.add_to_db(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    job.update_status(Status::Queued, &state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(job))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::loader::{Config, Service};
    use crate::routes::router::AppState;
    use axum::body::to_bytes;
    use axum::body::Body;
    use axum::{routing::post, Router};
    use http::{header, Request, StatusCode};
    use sqlx::SqlitePool;
    use std::fs;
    use std::io::Write;
    use std::path::PathBuf;
    use tempfile::tempdir;
    use tower::ServiceExt; // for `oneshot`
    use uuid::Uuid;

    // Helper function to initialize the database schema
    pub async fn init_db(pool: &SqlitePool) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
        CREATE TABLE IF NOT EXISTS jobs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id INTEGER NOT NULL,
            service TEXT NOT NULL,
            status TEXT NOT NULL,
            loc TEXT NOT NULL,
            dest_id TEXT,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )
    "#,
        )
        .execute(pool)
        .await?;
        Ok(())
    }

    // Helper functions to create multipart form data
    fn form_field(boundary: &str, name: &str, value: &str) -> Vec<u8> {
        format!(
            "--{boundary}\r\n\
                Content-Disposition: form-data; name=\"{name}\"\r\n\r\n\
                {value}\r\n"
        )
        .into_bytes()
    }

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
    fn form_text_file(boundary: &str, name: &str, filename: &str, content: &str) -> Vec<u8> {
        let mut part = format!(
            "--{boundary}\r\n\
                Content-Disposition: form-data; name=\"{name}\"; filename=\"{filename}\"\r\n\
                Content-Type: text/plain\r\n\r\n"
        )
        .into_bytes();
        part.extend_from_slice(content.as_bytes());
        part.extend_from_slice(b"\r\n");
        part
    }

    #[tokio::test]
    async fn test_upload() {
        // Setup the route
        let data_dir = tempdir().unwrap();
        let mut config = Config::new().unwrap();
        config.data_path = data_dir.path().to_str().unwrap().to_string();
        config.services = HashMap::from([(
            String::from("test-service"),
            Service {
                name: String::from("test-service"),
                upload_url: String::from("http://localhost/upload"),
                download_url: String::from("http://localhost/download"),
            },
        )]);

        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        init_db(&pool).await.unwrap(); // Initialize the database schema
        let state = AppState { pool, config };

        let app = Router::new()
            .route("/upload", post(upload))
            .with_state(state);

        // Create a multipart/form-data request
        let boundary = format!("----Boundary{}", Uuid::new_v4());
        let mut body = Vec::new();
        body.extend(form_field(&boundary, "service", "test-service"));
        body.extend(form_field(&boundary, "user_id", "42"));
        body.extend(form_file(
            &boundary,
            "file",
            "test.txt",
            "application/octet-stream",
            b"\x00\x01\x02\x03",
        ));
        body.extend(form_text_file(
            &boundary,
            "file",
            "test01.txt",
            "hello this is a test file",
        ));
        body.extend(format!("--{boundary}--\r\n").as_bytes());

        // Create the request
        let req = Request::builder()
            .method("POST")
            .uri("/upload")
            .header(
                header::CONTENT_TYPE,
                format!("multipart/form-data; boundary={boundary}"),
            )
            .body(Body::from(body))
            .unwrap();

        // Make the request
        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["id"], 1);
        assert_eq!(json["status"], String::from("Queued"));
        assert_eq!(json["service"], String::from("test-service"));
        assert_eq!(json["user_id"], 42);

        // Check if the file was saved correctly
        let expected_loc = json["loc"].as_str().unwrap();
        let expected_file = PathBuf::from(expected_loc).join("test.txt");
        assert!(expected_file.exists());
        let expected_file = PathBuf::from(expected_loc).join("test01.txt");
        assert!(expected_file.exists());
    }

    #[tokio::test]
    async fn test_upload_non_existing_service() {
        // Setup the route
        let data_dir = tempdir().unwrap();
        let mut config = Config::new().unwrap();
        config.data_path = data_dir.path().to_str().unwrap().to_string();

        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        init_db(&pool).await.unwrap(); // Initialize the database schema
        let state = AppState { pool, config };

        let app = Router::new()
            .route("/upload", post(upload))
            .with_state(state);

        // Create a multipart/form-data request
        let boundary = format!("----Boundary{}", Uuid::new_v4());
        let mut body = Vec::new();
        body.extend(form_field(&boundary, "service", "my-test-service"));
        body.extend(form_field(&boundary, "user_id", "42"));
        body.extend(form_file(
            &boundary,
            "file",
            "test.txt",
            "application/octet-stream",
            b"\x00\x01\x02\x03",
        ));
        body.extend(form_text_file(
            &boundary,
            "file",
            "test01.txt",
            "hello this is a test file",
        ));
        body.extend(format!("--{boundary}--\r\n").as_bytes());

        // Create the request
        let req = Request::builder()
            .method("POST")
            .uri("/upload")
            .header(
                header::CONTENT_TYPE,
                format!("multipart/form-data; boundary={boundary}"),
            )
            .body(Body::from(body))
            .unwrap();

        // Make the request
        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    }

    #[tokio::test]
    async fn test_download_non_init_db() {
        let config = Config::new().unwrap();
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap(); // Mock database connection;
        let state = State(AppState { pool, config }); // Mock state for testing
        let path = Path(1);
        let response = download(state, path).await;
        match response {
            Ok(_) => {}
            Err(e) => assert_eq!(e, StatusCode::INTERNAL_SERVER_ERROR),
        }
    }

    #[tokio::test]
    async fn test_download_non_existing_job() {
        let config = Config::new().unwrap();
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap(); // Mock database connection;
        init_db(&pool).await.unwrap(); // Initialize the database schema
        let state = State(AppState { pool, config }); // Mock state for testing
        let path = Path(1);
        let response = download(state, path).await;
        match response {
            Ok(_) => {}
            Err(e) => assert_eq!(e, StatusCode::NOT_FOUND),
        }
    }

    #[tokio::test]
    async fn test_download_completed_job() {
        let config = Config::new().unwrap();
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap(); // Mock database connection;
        init_db(&pool).await.unwrap(); // Initialize the database schema

        // Make a completed job
        let data_dir = tempdir().unwrap();
        let mut job = Job::new(data_dir.path().to_str().unwrap());
        fs::create_dir(&job.loc).unwrap(); // Create data directory
        let dummy_file_path = job.loc.join("output.zip");
        let mut file = fs::File::create(&dummy_file_path).unwrap();
        writeln!(file, "dummy data").unwrap(); // Create a dummy file
                                               //
        job.add_to_db(&pool).await.unwrap(); // Add job to the database;
        job.update_status(Status::Completed, &pool).await.unwrap(); // Update job status to Failed;

        let state = State(AppState { pool, config }); // Mock state for testing
        let path = Path(job.id);

        if let Ok(v) = download(state, path).await {
            assert_eq!(v, fs::read(dummy_file_path).unwrap());
        }
    }

    #[tokio::test]
    async fn test_download_failed_job() {
        let config = Config::new().unwrap();
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap(); // Mock database connection;
        init_db(&pool).await.unwrap(); // Initialize the database schema
        let mut job = Job::new("");
        job.add_to_db(&pool).await.unwrap(); // Add job to the database;
        job.update_status(Status::Failed, &pool).await.unwrap(); // Update job status to Failed;
                                                                 //
        let state = State(AppState { pool, config }); // Mock state for testing
        let path = Path(job.id);

        match download(state, path).await {
            Ok(_) => {}
            Err(e) => assert_eq!(e, StatusCode::NO_CONTENT),
        }
    }

    #[tokio::test]
    async fn test_download_cleaned_job() {
        let config = Config::new().unwrap();
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap(); // Mock database connection;
        init_db(&pool).await.unwrap(); // Initialize the database schema
        let mut job = Job::new("");
        job.add_to_db(&pool).await.unwrap(); // Add job to the database;
        job.update_status(Status::Cleaned, &pool).await.unwrap(); // Update job status to Cleaned;
                                                                  //
        let state = State(AppState { pool, config }); // Mock state for testing
        let path = Path(job.id);

        match download(state, path).await {
            Ok(_) => {}
            Err(e) => assert_eq!(e, StatusCode::NO_CONTENT),
        }
    }

    #[tokio::test]
    async fn test_download_accepted_job() {
        let config = Config::new().unwrap();
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap(); // Mock database connection;
        init_db(&pool).await.unwrap(); // Initialize the database schema
        let mut job = Job::new("");
        job.add_to_db(&pool).await.unwrap(); // Add job to the database;
        job.update_status(Status::Queued, &pool).await.unwrap(); // Update job status to Queued;
                                                                 //
        let state = State(AppState { pool, config }); // Mock state for testing
        let path = Path(job.id);

        match download(state, path).await {
            Ok(_) => {}
            Err(e) => assert_eq!(e, StatusCode::ACCEPTED),
        }
    }
}
