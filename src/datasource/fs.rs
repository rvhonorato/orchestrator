use crate::config::constants;

pub async fn init_fs() {
    match tokio::fs::create_dir(constants::UPLOADS_DIRECTORY).await {
        Ok(_) => tracing::info!("created uploads directory"),
        Err(_) => tracing::warn!("uploads directory exists - using it"),
    };
}
