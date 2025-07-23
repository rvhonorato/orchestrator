use crate::models::job_dao::Job;
use crate::models::status_dto::Status;
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
    // let mut user_data = None;

    let mut text_fields: HashMap<String, String> = HashMap::new();
    let mut job = Job::new(&state.config.data_path);

    // Collect all fields
    while let Ok(Some(field)) = multipart.next_field().await {
        if let Some(field_name) = field.name() {
            let field_name = field_name.to_string();

            if field.file_name().is_some() {
                // Handle file
                // NOTE: This is expecting filenames to be safe!!
                let filename = field.file_name().unwrap().to_string();
                job.save_to_disk(field, &filename).await?;
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

    let user_id = match text_fields.get("user_id") {
        Some(id) => id
            .parse::<i32>()
            .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid user_id".to_string()))?,
        None => return Err((StatusCode::BAD_REQUEST, "Missing user_id".to_string())),
    };
    let service_id = match text_fields.get("service") {
        Some(id) => {
            // TODO: Check if service is valid if not - clean
            id.clone()
        }
        None => return Err((StatusCode::BAD_REQUEST, "Missing service_id".to_string())),
    };

    job.set_user_id(user_id);
    job.set_service(service_id);

    // Add it to the database and handle potential errors
    job.add_to_db(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let _ = job.update_status(Status::Queued, &state.pool).await;

    Ok(Json(job))
}

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

    #[tokio::test]
    async fn test_upload() {
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
        body.extend(format!("--{boundary}--\r\n").as_bytes());

        // Create the request
        let req = Request::builder()
            .method("POST")
            .uri("/upload")
            .header(
                header::CONTENT_TYPE,
                format!("multipart/form-data; boundary={}", boundary),
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
        assert_eq!(json["service"], "my-test-service");
        assert_eq!(json["user_id"], 42);

        // Check if the file was saved correctly
        let expected_loc = json["loc"].as_str().unwrap();
        let expected_file = PathBuf::from(expected_loc).join("test.txt");
        assert!(expected_file.exists());
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
