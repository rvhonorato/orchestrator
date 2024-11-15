use crate::controllers;
use axum::{
    routing::{get, post},
    Router,
};
use tower_http::trace;
use tower_http::trace::TraceLayer;
use tracing::Level;

pub fn create_routes() -> Router {
    Router::new()
        .route("/", get(controllers::ping))
        .route("/upload", post(controllers::upload))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(trace::DefaultMakeSpan::new().level(Level::INFO))
                .on_response(trace::DefaultOnResponse::new().level(Level::INFO)),
        )
}
