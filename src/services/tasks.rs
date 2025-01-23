use sqlx::SqlitePool;
use tracing::info;

use crate::models::{queue_dao::Queue, status_dto::Status};

pub async fn scheduler(pool: SqlitePool) {
    let mut queue = Queue::new();
    let status = Status::from_string("queued");
    let _ = queue.load(status, &pool).await;
    info!("{:?}", queue);
    // TODO: add logic to submit the jobs
}
