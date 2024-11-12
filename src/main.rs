use std::net::SocketAddr;

use axum::{routing, Router};
use tokio::net::TcpListener;
use tower_http::trace::{self, TraceLayer};
use tracing::Level;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_target(false)
        .compact()
        .init();

    let app = Router::new().route("/", routing::get(root)).layer(
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

async fn root() -> &'static str {
    "Hello!"
}
