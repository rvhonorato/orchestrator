use crate::config::loader::Config;
use crate::controllers::client::{retrieve, submit};
use crate::controllers::orchestrator::__path_download;
use crate::controllers::orchestrator::__path_upload;
use crate::controllers::orchestrator::{download, upload};
use crate::controllers::ping::ping;
use crate::models::job_dao::Job;
use axum::extract::DefaultBodyLimit;
use axum::{
    routing::{get, post},
    Router,
};
use sqlx::SqlitePool;
use tower_http::trace;
use tower_http::trace::TraceLayer;
use tracing::Level;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
    pub config: Config,
}

#[derive(OpenApi)]
#[openapi(
    paths(
        upload,
        download
    ),
    components(
        schemas(Job)
    ),
    tags(
        (name = "files", description = "File management endpoints")
    )
)]
struct ApiDoc;

pub fn create_routes(pool: SqlitePool, config: Config) -> Router {
    let state = AppState { pool, config };
    Router::new()
        .route("/", get(ping))
        .route("/upload", post(upload))
        .route("/download/{id}", get(download))
        .merge(SwaggerUi::new("/swagger").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .with_state(state)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(trace::DefaultMakeSpan::new().level(Level::INFO))
                .on_response(trace::DefaultOnResponse::new().level(Level::INFO)),
        )
        // .layer(DefaultBodyLimit::disable())
        .layer(DefaultBodyLimit::max(400 * 1024 * 1024)) // Set max body size to 400MB
}

pub fn create_client_routes(pool: SqlitePool, config: Config) -> Router {
    let state = AppState { pool, config };
    Router::new()
        .route("/", get(ping))
        .route("/submit", post(submit))
        .route("/retrieve/{id}", get(retrieve))
        .with_state(state)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(trace::DefaultMakeSpan::new().level(Level::DEBUG))
                .on_response(trace::DefaultOnResponse::new().level(Level::DEBUG)),
        )
        .layer(DefaultBodyLimit::disable())
}
