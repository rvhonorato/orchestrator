use std::env;

pub async fn init_fs() {
    let wd_path = env::var("ORCHESTRATOR_DATA_PATH").expect("ORCHESTRATOR_DATA_PATH not defined");
    let upload_path = format!("{}/uploads", wd_path);
    match tokio::fs::create_dir(upload_path).await {
        Ok(_) => tracing::info!("created uploads directory"),
        Err(_) => tracing::warn!("uploads directory exists - using it"),
    };
}
