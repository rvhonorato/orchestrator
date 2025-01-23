use crate::controllers::orchestrator::{download, upload};
use crate::controllers::ping::ping;
use axum::{
    routing::{get, post},
    Router,
};
use sqlx::SqlitePool;
use tower_http::trace;
use tower_http::trace::TraceLayer;
use tracing::Level;
#[derive(Clone)]
struct AppState {
    db: SqlitePool,
}
pub fn create_routes(pool: SqlitePool) -> Router {
    let state = AppState { db: pool };
    Router::new()
        .route("/", get(ping))
        .route("/upload", post(upload))
        .route("/download/{id}", get(download))
        .with_state(state.db)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(trace::DefaultMakeSpan::new().level(Level::INFO))
                .on_response(trace::DefaultOnResponse::new().level(Level::INFO)),
        )
}
