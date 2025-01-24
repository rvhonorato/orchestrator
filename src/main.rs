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
use services::tasks::{getter, sender};
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tokio_schedule::{every, Job};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize a logger
    tracing_subscriber::fmt()
        .with_target(false)
        // .with_max_level(tracing::Level::DEBUG)
        .compact()
        .init();

    // Initialize the database
    let pool = init_db().await;

    // Initialize the filesystem
    let _ = init_fs().await;

    // Create a scheduled job
    let sender_task = every(500).millisecond().perform(|| {
        let pool_clone = pool.clone();
        async move { sender(pool_clone).await }
    });

    let getter_task = every(500).millisecond().perform(|| {
        let pool_clone = pool.clone();
        async move { getter(pool_clone).await }
    });

    // Create app
    let app = create_routes(pool.clone());

    // Initialize socket
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::info!("listening on {}", addr);

    let listener = TcpListener::bind(addr).await?;

    tokio::select! {
        _ = sender_task => {},
        _ = getter_task => {},
        _ = axum::serve(listener, app.into_make_service()) => {},
    }

    Ok(())
}
