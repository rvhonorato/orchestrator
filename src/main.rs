mod controllers;
mod models;
mod routes;
mod services;
mod utils;
use crate::models::job_dao::UPLOADS_DIRECTORY;
use crate::models::job_dto::create_jobs_table;
use crate::routes::routes::create_routes;
use sqlx::SqlitePool;
use std::net::SocketAddr;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_target(false)
        .compact()
        .init();

    let pool = SqlitePool::connect("sqlite::memory:")
        .await
        .unwrap_or_else(|e| panic!("Database connection failed: {}", e));

    create_jobs_table(&pool)
        .await
        .expect("failed to create the jobs table");

    match tokio::fs::create_dir(UPLOADS_DIRECTORY).await {
        Ok(_) => tracing::info!("created uploads directory"),
        Err(_) => tracing::warn!("uploads directory exists - using it"),
    };

    let app = create_routes(pool);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::info!("listening on {}", addr);

    let listener = TcpListener::bind(addr).await?;

    axum::serve(listener, app.into_make_service()).await?;

    Ok(())
}
