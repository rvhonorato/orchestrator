use axum::{
    body::Bytes,
    extract::{Json, Multipart},
    http::StatusCode,
    routing::{get, post},
    BoxError, Router,
};
use futures::{Stream, TryStreamExt};
use serde::{Deserialize, Serialize};
use std::io;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tokio::{fs::File, io::BufWriter};
use tokio_util::io::StreamReader;
use tower_http::trace::{self, TraceLayer};
use tracing::Level;
use uuid::Uuid;

#[derive(Deserialize)]
struct UploadPayload {
    user_id: i32,
    // service: String,
    // access_level: u8,
}

#[derive(Serialize)]
struct Job {
    user_id: i32,
    job_id: Uuid,
}

#[derive(Serialize)]
struct Ping {
    message: String,
}

async fn upload(mut multipart: Multipart) -> Result<Json<Job>, (StatusCode, String)> {
    let mut user_data = None;

    while let Ok(Some(field)) = multipart.next_field().await {
        if let Some(field_name) = field.name() {
            // check if this field name is file before proceeding
            if field_name == "file" {
                // TODO: use a proper filename here
                let file_name = Uuid::new_v4().to_string();

                stream_to_file(&file_name, field).await?;
            } else if field_name == "data" {
                let data = field
                    .text()
                    .await
                    .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
                user_data = Some(
                    serde_json::from_str::<UploadPayload>(&data)
                        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?,
                );
            }
        }
    }
    let user_data = user_data.ok_or((StatusCode::BAD_REQUEST, "Missing JSON data".to_string()))?;

    let mut j = submit().await;
    j.user_id = user_data.user_id;
    Ok(Json(j))
}

async fn stream_to_file<S, E>(path: &str, stream: S) -> Result<(), (StatusCode, String)>
where
    S: Stream<Item = Result<Bytes, E>>,
    E: Into<BoxError>,
{
    if !path_is_valid(path) {
        return Err((StatusCode::BAD_REQUEST, "Invalid path".to_owned()));
    }

    async {
        // Convert the stream into an AsyncRead.
        let body_with_io_error = stream.map_err(|err| io::Error::new(io::ErrorKind::Other, err));
        let body_reader = StreamReader::new(body_with_io_error);
        futures::pin_mut!(body_reader);

        // Create the file. File implements AsyncWrite.
        let path = std::path::Path::new(UPLOADS_DIRECTORY).join(path);
        let mut file = BufWriter::new(File::create(path).await?);

        // Copy the body into the file.
        tokio::io::copy(&mut body_reader, &mut file).await?;

        Ok::<_, io::Error>(())
    }
    .await
    .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))
}

// to prevent directory traversal attacks we ensure the path consists of exactly one normal
// component
fn path_is_valid(path: &str) -> bool {
    let path = std::path::Path::new(path);
    let mut components = path.components().peekable();

    if let Some(first) = components.peek() {
        if !matches!(first, std::path::Component::Normal(_)) {
            return false;
        }
    }

    components.count() == 1
}

async fn submit() -> Job {
    // Placeholder
    Job {
        user_id: 0,
        job_id: Uuid::new_v4(),
    }
}

async fn ping() -> Json<Ping> {
    Json(Ping {
        message: "pong".to_string(),
    })
}

const UPLOADS_DIRECTORY: &str = "uploads";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_target(false)
        .compact()
        .init();

    match tokio::fs::create_dir(UPLOADS_DIRECTORY).await {
        Ok(_) => tracing::info!("created uploads directory"),
        Err(_) => tracing::warn!("uploads directory exists - using it"),
    };

    let app = Router::new()
        .route("/", get(ping))
        .route("/upload", post(upload))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(trace::DefaultMakeSpan::new().level(Level::INFO))
                .on_response(trace::DefaultOnResponse::new().level(Level::INFO)),
        );

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::info!("listening on {}", addr);

    let listener = TcpListener::bind(addr).await?;

    axum::serve(listener, app.into_make_service()).await?;

    Ok(())
}
