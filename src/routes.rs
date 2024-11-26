use crate::controllers;
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
        .route("/", get(controllers::ping))
        .route("/upload", post(controllers::upload))
        .with_state(state.db)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(trace::DefaultMakeSpan::new().level(Level::INFO))
                .on_response(trace::DefaultOnResponse::new().level(Level::INFO)),
        )
}
