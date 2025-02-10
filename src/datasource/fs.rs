pub async fn init_fs(data_path: &str) {
    match tokio::fs::create_dir(data_path).await {
        Ok(_) => tracing::info!("created uploads directory"),
        Err(_) => tracing::warn!("uploads directory exists - using it"),
    };
}
