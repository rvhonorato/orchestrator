use std::path::PathBuf;

use super::{queue_dao::Queue, status_dto::Status};
use crate::models::job_dao::Job;
use sqlx::{Row, SqlitePool};

impl Queue {
    pub async fn load(&mut self, status: Status, pool: &SqlitePool) -> Result<(), sqlx::Error> {
        let rows = sqlx::query("SELECT * FROM jobs WHERE status = ?")
            .bind(status.to_string())
            .fetch_all(pool)
            .await?;

        let jobs: Vec<Job> = rows
            .into_iter()
            .map(|row| {
                let status: String = row.get("status");
                let loc: String = row.get("loc");
                Job {
                    id: row.get("id"),
                    user_id: row.get("user_id"),
                    status: Status::from_string(&status),
                    loc: PathBuf::from(loc),
                    dest_id: row.get("dest_id"),
                }
            })
            .collect();
        self.jobs = jobs;
        Ok(())
    }
}
