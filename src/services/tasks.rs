use crate::models::{queue_dao::Queue, status_dto::Status};
use crate::services::orchestrator;
use sqlx::SqlitePool;
use tracing::{debug, error, info};

use super::orchestrator::{Destinations, DownloadError};

pub async fn sender(pool: SqlitePool) {
    let mut queue = Queue::new();
    if queue.load(Status::Queued, &pool).await.is_ok() {
        let futures = queue
            .jobs
            .into_iter()
            .map(|mut j| {
                let pool_clone = pool.clone();
                tokio::spawn(async move {
                    j.update_status(Status::Processing, &pool_clone).await.ok();

                    match orchestrator::send(&j, orchestrator::Destinations::Jobd).await {
                        Ok(upload_id) => {
                            info!("submitting: {:?}", j);
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

pub async fn getter(pool: SqlitePool) {
    let mut queue = Queue::new();
    if queue.load(Status::Processing, &pool).await.is_ok() {
        let futures = queue
            .jobs
            .into_iter()
            .map(|mut j| {
                let pool_clone = pool.clone();
                tokio::spawn(async move {
                    let result = orchestrator::retrieve(&j, Destinations::Jobd).await;
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
