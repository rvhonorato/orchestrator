use std::fs;
use std::time::SystemTime;

use crate::config::loader::Config;
use crate::models::job_dao::Job;
use crate::models::queue_dao::PayloadQueue;
use crate::models::{queue_dao::Queue, status_dto::Status};
use crate::services::orchestrator;
use sqlx::SqlitePool;
use tracing::info;
use tracing::{debug, error};

use super::client::ClientError;
use super::jobd::Jobd;
use super::orchestrator::DownloadError;

pub async fn cleaner(pool: SqlitePool, config: Config) {
    // List all directories inside the config.data_path
    let elements = match fs::read_dir(&config.data_path) {
        Ok(e) => e,
        Err(_) => {
            error!("could not read directory: {}", config.data_path);
            return;
        }
    };

    let futures = elements.into_iter().map(|entry| async {
        let entry = match entry {
            Ok(d) => d,
            Err(_) => {
                error!("could not read subdir");
                return;
            }
        };
        let path = entry.path();
        if !path.is_dir() {
            return;
        }
        let metadata = match fs::metadata(&path) {
            Ok(m) => m,
            Err(_) => {
                error!("could not read metadata");
                return;
            }
        };
        if let Ok(mod_time) = metadata.modified() {
            let current_time = SystemTime::now();
            if let Ok(age) = current_time.duration_since(mod_time) {
                if age >= config.max_age {
                    debug!(
                        "{:?} - {:?} - {:?}",
                        path.display(),
                        age.as_secs(),
                        config.max_age
                    );
                    let mut job = Job::new("");
                    match job.retrieve_by_loc(path.display().to_string(), &pool).await {
                        Ok(_) => {
                            let _ = job.update_status(Status::Cleaned, &pool).await;
                            if let Err(e) = job.remove_from_disk() {
                                error!("error: {:?} - could not remove {:?}", e, path)
                            }
                        }
                        Err(e) => error!("{:?} - not found: {:?}", e, path),
                    }
                }
            }
        };
    });

    futures::future::join_all(futures).await;
}

pub async fn sender(pool: SqlitePool, config: Config) {
    let mut queue = Queue::new(&config);
    if queue.load(&pool).await.is_ok() {
        // info!("There are {:?} queued jobs", queue.jobs.len());
        let futures = queue
            .jobs
            .into_iter()
            .map(|mut j| {
                // info!("{:?}", j);
                let pool_clone = pool.clone();
                let config_clone = config.clone();
                tokio::spawn(async move {
                    j.update_status(Status::Processing, &pool_clone).await.ok();

                    match orchestrator::send(&j, &config_clone, Jobd).await {
                        Ok(upload_id) => {
                            info!("submitting: {:?}", j);
                            j.update_status(Status::Submitted, &pool_clone).await.ok();
                            j.update_dest_id(upload_id, &pool_clone).await.ok();
                            // debug!("{:?}", j);
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
    let mut queue = Queue::new(&config);
    if queue
        .list_per_status(Status::Submitted, &pool)
        .await
        .is_ok()
    {
        // info!("There are {:?} submitted jobs", queue.jobs.len());
        let futures = queue
            .jobs
            .into_iter()
            .map(|mut j| {
                let pool_clone = pool.clone();
                let config_clone = config.clone();
                tokio::spawn(async move {
                    let result = orchestrator::retrieve(&j, &config_clone, Jobd).await;
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

pub async fn runner(pool: SqlitePool, config: Config) {
    let mut queue = PayloadQueue::new(&config);
    if queue.list_per_status(Status::Prepared, &pool).await.is_ok() {
        let futures = queue
            .jobs
            .into_iter()
            .map(|mut j| {
                let pool_clone = pool.clone();
                // let config_clone = config.clone();
                tokio::spawn(async move {
                    let result = j.execute();
                    match result {
                        Ok(_) => {
                            j.update_status(Status::Completed, &pool_clone).await.ok();
                        }
                        Err(ClientError::ExecutionError) => {
                            todo!("handle")
                        }
                    }
                })
            })
            .collect::<Vec<_>>();

        futures::future::join_all(futures).await;
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use crate::config::loader::{Config, Service};
    use crate::models::{job_dao::Job, job_dto::create_jobs_table};
    use std::{path::Path, time::Duration};
    use tempfile::TempDir;
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_sender() {
        let pool = SqlitePool::connect(":memory:")
            .await
            .unwrap_or_else(|e| panic!("Database connection failed: {e}"));
        let mut config = Config::new().unwrap();
        config.services.insert(
            "A".to_string(),
            Service {
                name: "A".to_string(),
                upload_url: "http://example.com/upload_a".to_string(),
                download_url: "http://example.com/download_a".to_string(),
                runs_per_user: 5,
            },
        );

        create_jobs_table(&pool).await.unwrap();

        // add a job
        let tempdir = TempDir::new().unwrap();
        let mut job = Job::new(tempdir.path().to_str().unwrap());
        job.set_service("A".to_string());
        job.add_to_db(&pool).await.unwrap();
        job.update_status(Status::Queued, &pool).await.unwrap();
        let id = job.id;

        sender(pool.clone(), config).await;

        let tempdir = TempDir::new().unwrap();
        let mut _job = Job::new(tempdir.path().to_str().unwrap());
        _job.retrieve_id(id, &pool).await.unwrap();

        // Since nothing is configured, it will fail
        //  the only thing we need to test here is if
        //  the status is being updated
        assert_eq!(_job.status, Status::Failed);

        // TODO: Add mock the `send` function to test the match arm
    }

    #[tokio::test]
    async fn test_getter() {
        let pool = SqlitePool::connect(":memory:")
            .await
            .unwrap_or_else(|e| panic!("Database connection failed: {e}"));
        let config = Config::new().unwrap();

        create_jobs_table(&pool).await.unwrap();

        // add a job
        let tempdir = TempDir::new().unwrap();
        let mut job = Job::new(tempdir.path().to_str().unwrap());
        job.add_to_db(&pool).await.unwrap();
        job.update_status(Status::Submitted, &pool).await.unwrap();
        let id = job.id;

        getter(pool.clone(), config).await;

        let tempdir = TempDir::new().unwrap();
        let mut _job = Job::new(tempdir.path().to_str().unwrap());
        _job.retrieve_id(id, &pool).await.unwrap();

        // Since nothing is configured, it will fail
        //  the only thing we need to test here is if
        //  the status is not being updated
        assert_eq!(_job.status, Status::Submitted);

        // TODO: Add mock the `retrieve` function to test the match arm
    }

    #[tokio::test]
    async fn test_cleaner() {
        let pool = SqlitePool::connect(":memory:")
            .await
            .unwrap_or_else(|e| panic!("Database connection failed: {e}"));
        let mut config = Config::new().unwrap();

        create_jobs_table(&pool).await.unwrap();

        // add a job
        let tempdir = TempDir::new().unwrap();
        let mut job = Job::new(tempdir.path().to_str().unwrap());
        fs::create_dir_all(&job.loc).unwrap();

        job.add_to_db(&pool).await.unwrap();

        config.max_age = Duration::from_nanos(1);
        config.data_path = tempdir.path().to_str().unwrap().to_string();

        // Sleep to allow the file to age
        // NOTE: This is simpler than editing the mtime
        sleep(Duration::from_nanos(1)).await;

        assert!(Path::new(&job.loc).exists());

        cleaner(pool.clone(), config).await;

        assert!(!Path::new(&job.loc).exists());

        let mut _job = Job::new("");
        let _ = _job.retrieve_id(job.id, &pool).await;

        assert_eq!(_job.status, Status::Cleaned);
    }

    // TODO: Implement test for runner
    // #[tokio::test]
    // async fn test_runner() {
    //     todo!()
    // }
}
