use std::path::PathBuf;

use super::{queue_dao::Queue, status_dto::Status};
use crate::models::job_dao::Job;
use sqlx::{Row, SqlitePool};
use std::collections::HashMap;

impl Queue<'_> {
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
    pub async fn load(&mut self, pool: &SqlitePool) -> Result<(), sqlx::Error> {
        let query = r#"
            -- Combined query: Get queued jobs with their respective submitted job counts
            -- Uses CTE to precompute submitted counts per user/service, then joins with queued jobs
            WITH submitted_counts AS (
                SELECT user_id, service, COUNT(*) as count
                FROM jobs
                WHERE status = 'submitted'
                GROUP BY user_id, service
            )
            SELECT
                j.id, j.user_id, j.service, j.status, j.loc, j.dest_id,
                COALESCE(sc.count, 0) as submitted_count  -- Default to 0 if no submitted jobs
            FROM jobs j
            LEFT JOIN submitted_counts sc ON j.user_id = sc.user_id AND j.service = sc.service
            WHERE j.status = ?  -- Parameterized for queued jobs
            ORDER BY j.user_id, j.service, j.id  -- Group related records for efficient processing
        "#;

        let rows = sqlx::query(query)
            .bind(Status::Queued.to_string())
            .fetch_all(pool)
            .await?;

        // Precompute limits
        let service_limits: HashMap<&str, u16> = self
            .config
            .services
            .iter()
            .map(|(service, config)| (service.as_str(), config.runs_per_user))
            .collect();

        // Process jobs
        let mut jobs = Vec::new();
        let mut current_key = None;
        let mut current_count = 0;
        let mut current_limit = 0;

        for row in rows {
            let user_id: i64 = row.get("user_id");
            let service: String = row.get("service");
            let submitted_count: i64 = row.get("submitted_count");

            let key = (user_id, service);
            let limit = service_limits.get(key.1.as_str()).copied().unwrap();

            // Reset counter when key changes
            if Some(&key) != current_key.as_ref() {
                current_key = Some(key.clone());
                current_count = 0;
                current_limit = (limit as i64 - submitted_count).max(0) as usize;
            }

            // Add job if under limit
            if current_count < current_limit {
                let status: String = row.get("status");
                let loc: String = row.get("loc");

                jobs.push(Job {
                    id: row.get("id"),
                    user_id: user_id.try_into().unwrap(),
                    service: key.1.clone(),
                    status: Status::from_string(&status),
                    loc: PathBuf::from(loc),
                    dest_id: row.get("dest_id"),
                });

                current_count += 1;
            }
        }

        self.jobs = jobs;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::loader::{Config, Service};
    use crate::models::job_dto::create_jobs_table;

    #[tokio::test]
    async fn test_load_limits_jobs_per_user_per_service() {
        // Setup in-memory SQLite database
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
        config.services.insert(
            "B".to_string(),
            Service {
                name: "B".to_string(),
                upload_url: "http://example.com/upload_b".to_string(),
                download_url: "http://example.com/download_b".to_string(),
                runs_per_user: 5,
            },
        );
        config.services.insert(
            "C".to_string(),
            Service {
                name: "C".to_string(),
                upload_url: "http://example.com/upload_c".to_string(),
                download_url: "http://example.com/download_c".to_string(),
                runs_per_user: 1,
            },
        );

        create_jobs_table(&pool).await.unwrap();

        // Insert 5 submitted jobs for user 1 - service A
        for _ in 0..5 {
            sqlx::query("INSERT INTO jobs (user_id, service, status, loc, dest_id) VALUES (1, 'A', 'submitted', 'loc', NULL)")
                .execute(&pool).await.unwrap();
        }
        // Insert 2 queued jobs for user 1 - service A
        for _ in 0..2 {
            sqlx::query("INSERT INTO jobs (user_id, service, status, loc, dest_id) VALUES (1, 'A', 'queued', 'loc', NULL)")
                .execute(&pool).await.unwrap();
        }

        // Insert 3 submitted jobs for user 1 - service B
        for _ in 0..3 {
            sqlx::query("INSERT INTO jobs (user_id, service, status, loc, dest_id) VALUES (1, 'B', 'submitted', 'loc', NULL)")
                .execute(&pool).await.unwrap();
        }
        // Insert 3 queued jobs for user 1 - service B
        for _ in 0..3 {
            sqlx::query("INSERT INTO jobs (user_id, service, status, loc, dest_id) VALUES (1, 'B', 'queued', 'loc', NULL)")
                .execute(&pool).await.unwrap();
        }

        // Here user 1 has:
        //  - 5 submitted / 2 queued jobs for service A
        //  - 3 submitted / 3 queued jobs for service B

        // Load the queue
        let mut queue = Queue::new(&config);
        queue.load(&pool).await.unwrap();

        // User already has 5 submitted for service A,
        // > no more queued jobs for service A should be loaded
        let jobs_for_a = queue.jobs.iter().filter(|j| j.service == "A").count();
        let expected_a = 0;
        assert_eq!(jobs_for_a, expected_a,);
        // User has 3 submitted and 3 queued for service B,
        // > 2 more queued jobs for service B should be loaded, since max is 5
        let jobs_for_b = queue.jobs.iter().filter(|j| j.service == "B").count();
        let expected_b = 2;
        assert_eq!(jobs_for_b, expected_b,);

        // Add more jobs for another user
        for _ in 0..2 {
            sqlx::query("INSERT INTO jobs (user_id, service, status, loc, dest_id) VALUES (2, 'A', 'queued', 'loc', NULL)")
                .execute(&pool).await.unwrap();
        }

        // Reload the queue
        queue.load(&pool).await.unwrap();

        // Now there should be 2 jobs for user 2 - service A
        let jobs_for_a = queue.jobs.iter().filter(|j| j.service == "A").count();
        let expected_a = 2;
        assert_eq!(jobs_for_a, expected_a,);

        // Add jobs for service C, which has a limit of 1
        // Add two queued jobs for user 1 - service C
        for _ in 0..2 {
            sqlx::query("INSERT INTO jobs (user_id, service, status, loc, dest_id) VALUES (1, 'C', 'queued', 'loc', NULL)")
                .execute(&pool).await.unwrap();
        }
        // Add two queued jobs for user 2 - service C
        for _ in 0..2 {
            sqlx::query("INSERT INTO jobs (user_id, service, status, loc, dest_id) VALUES (2, 'C', 'queued', 'loc', NULL)")
                .execute(&pool).await.unwrap();
        }
        // Reload the queue
        queue.load(&pool).await.unwrap();

        // Since the limit for service C per user is 1, there should be two jobs loaded in total
        let jobs_for_c = queue.jobs.iter().filter(|j| j.service == "C").count();
        let expected_c = 2;
        assert_eq!(jobs_for_c, expected_c);
    }
}
