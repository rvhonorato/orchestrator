use crate::config::loader::Config;
use crate::models::{queue_dao::Queue, status_dto::Status};
use crate::services::orchestrator;
use sqlx::SqlitePool;
use tracing::{debug, error};

use super::orchestrator::DownloadError;

pub async fn sender(pool: SqlitePool, config: Config) {
    let mut queue = Queue::new();
    if queue.load(Status::Queued, &pool).await.is_ok() {
        // info!("{:?}", queue.jobs.len());
        let futures = queue
            .jobs
            .into_iter()
            .map(|mut j| {
                // info!("{:?}", j);
                let pool_clone = pool.clone();
                let config_clone = config.clone();
                tokio::spawn(async move {
                    j.update_status(Status::Processing, &pool_clone).await.ok();

                    match orchestrator::send(&j, &config_clone).await {
                        Ok(upload_id) => {
                            // info!("submitting: {:?}", j);
                            j.update_status(Status::Submitted, &pool_clone).await.ok();
                            j.update_dest_id(upload_id, &pool_clone).await.ok();
                            debug!("{:?}", j);
                        }
                        Err(orchestrator::UploadError::UnexpectedStatus(status)) => {
                            error!("Unexpected status: {:?}", status);
                        }
                        Err(e) => {
                            error!("Upload error: {:?}", e);
                            j.update_status(Status::Failed, &pool_clone).await.ok();
                        }
                    }
                })
            })
            .collect::<Vec<_>>();

        futures::future::join_all(futures).await;
    }
}

pub async fn getter(pool: SqlitePool, config: Config) {
    let mut queue = Queue::new();
    if queue
        .list_per_status(Status::Submitted, &pool)
        .await
        .is_ok()
    {
        let futures = queue
            .jobs
            .into_iter()
            .map(|mut j| {
                let pool_clone = pool.clone();
                let config_clone = config.clone();
                tokio::spawn(async move {
                    let result = orchestrator::retrieve(&j, &config_clone).await;
                    match result {
                        Ok(_) => {
                            j.update_status(Status::Completed, &pool_clone).await.ok();
                        }
                        Err(DownloadError::NotReady) => {}
                        Err(DownloadError::NotFound) => {
                            j.update_status(Status::Unknown, &pool_clone).await.ok();
                        }
                        Err(e) => {
                            error!("{:?}", e);
                        }
                    }
                })
            })
            .collect::<Vec<_>>();

        futures::future::join_all(futures).await;
    }
}
