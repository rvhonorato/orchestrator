use std::path::PathBuf;

use super::{queue_dao::Queue, status_dto::Status};
use crate::models::job_dao::Job;
use sqlx::{Row, SqlitePool};

impl Queue {
    pub async fn list_per_status(
        &mut self,
        status: Status,
        pool: &SqlitePool,
    ) -> Result<(), sqlx::Error> {
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
                    service: row.get("service"),
                    status: Status::from_string(&status),
                    loc: PathBuf::from(loc),
                    dest_id: row.get("dest_id"),
                }
            })
            .collect();
        self.jobs = jobs;
        Ok(())
    }
    pub async fn load(&mut self, status: Status, pool: &SqlitePool) -> Result<(), sqlx::Error> {
        //=======================================================================
        // This query selects jobs with the specified status, ensuring:
        // 1. Each user has no more than 4 jobs in 'Processing' status
        // 2. For each user, only up to 4 jobs are selected
        //
        // The query uses a Common Table Expression (CTE) to:
        // - Count processing jobs per user
        // - Assign a rank to jobs within each user's set of jobs
        // - Select only jobs where:
        //   a) The user has fewer than 4 processing jobs
        //   b) The job is within the first 4 jobs for that user
        //=======================================================================
        let rows = sqlx::query(
            "WITH UserJobCounts AS (
                SELECT 
                    j.*, 
                    (SELECT COUNT(*) FROM jobs 
                    WHERE user_id = j.user_id AND status = 'submitted') AS submitted_count,
                    ROW_NUMBER() OVER (PARTITION BY j.user_id ORDER BY j.id) AS user_job_rank
                FROM jobs j
                WHERE j.status = ?
            )
            SELECT * FROM UserJobCounts
            WHERE submitted_count < 4 AND user_job_rank <= 4",
        )
        .bind(status.to_string())
        .fetch_all(pool)
        .await?;
        //=======================================================================

        let jobs: Vec<Job> = rows
            .into_iter()
            .map(|row| {
                let status: String = row.get("status");
                let loc: String = row.get("loc");
                Job {
                    id: row.get("id"),
                    user_id: row.get("user_id"),
                    service: row.get("service"),
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
