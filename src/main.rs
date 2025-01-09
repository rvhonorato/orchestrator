mod controllers;
mod datasource;
mod models;
mod routes;
mod services;
mod utils;
use crate::datasource::db::init_db;
use crate::datasource::fs::init_fs;
use crate::routes::routes::create_routes;
use std::net::SocketAddr;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_target(false)
        .compact()
        .init();

    let pool = init_db().await;

    let _ = init_fs().await;

    let app = create_routes(pool);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::info!("listening on {}", addr);

    let listener = TcpListener::bind(addr).await?;

    axum::serve(listener, app.into_make_service()).await?;

    Ok(())
}
