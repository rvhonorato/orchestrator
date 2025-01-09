mod config;
mod controllers;
mod datasource;
mod models;
mod routes;
mod services;
mod utils;
use crate::datasource::db::init_db;
use crate::datasource::fs::init_fs;
use crate::routes::router::create_routes;
use std::net::SocketAddr;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize a logger
    tracing_subscriber::fmt()
        .with_target(false)
        .compact()
        .init();

    // Initialize the database
    let pool = init_db().await;

    // Initialize the filesystem
    let _ = init_fs().await;

    // Create app
    let app = create_routes(pool);

    // Initialize socket
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::info!("listening on {}", addr);

    let listener = TcpListener::bind(addr).await?;

    // Serve the app
    axum::serve(listener, app.into_make_service()).await?;

    Ok(())
}
