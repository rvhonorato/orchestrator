use crate::services::services::UPLOADS_DIRECTORY;

pub async fn init_fs() {
    match tokio::fs::create_dir(UPLOADS_DIRECTORY).await {
        Ok(_) => tracing::info!("created uploads directory"),
        Err(_) => tracing::warn!("uploads directory exists - using it"),
    };
}
