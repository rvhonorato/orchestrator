use std::path::PathBuf;

use crate::models::job_dao::Job;
use crate::models::status_dto::Status;
use sqlx::{Row, SqlitePool};

pub async fn create_jobs_table(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS jobs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id INTEGER NOT NULL,
            service TEXT NOT NULL,
            status TEXT NOT NULL,
            loc TEXT NOT NULL,
            dest_id TEXT,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )
    "#,
    )
    .execute(pool)
    .await?;
    Ok(())
}

impl Job {
    pub async fn add_to_db(&mut self, pool: &SqlitePool) -> Result<(), sqlx::Error> {
        let result =
            sqlx::query("INSERT INTO jobs (user_id, loc, status, service) VALUES (?, ?, ?, ?)")
                .bind(self.user_id)
                .bind(self.loc.to_str())
                .bind(self.status.to_string())
                .bind(self.service.to_string())
                .execute(pool)
                .await?;

        let job_id = result.last_insert_rowid();
        self.id = job_id as i32;

        Ok(())
    }

    pub async fn update_status(
        &mut self,
        status: Status,
        pool: &SqlitePool,
    ) -> Result<(), sqlx::Error> {
        let _result = sqlx::query("UPDATE jobs SET status = ? WHERE id = ?")
            .bind(status.to_string())
            .bind(self.id)
            .execute(pool)
            .await?;

        self.status = status;

        Ok(())
    }

    pub async fn update_dest_id(
        &mut self,
        dest_id: String,
        pool: &SqlitePool,
    ) -> Result<(), sqlx::Error> {
        let _result = sqlx::query("UPDATE jobs SET dest_id = ? WHERE id = ?")
            .bind(&dest_id)
            .bind(self.id)
            .execute(pool)
            .await?;

        self.dest_id = dest_id;

        Ok(())
    }

    pub async fn retrieve_id(&mut self, id: i32, pool: &SqlitePool) -> Result<(), sqlx::Error> {
        let result = sqlx::query("SELECT * FROM jobs WHERE id = ?")
            .bind(id)
            .fetch_optional(pool)
            .await?;
        if let Some(row) = result {
            let status: String = row.get("status");
            let loc: String = row.get("loc");

            self.id = row.get("id");
            self.user_id = row.get("user_id");
            self.status = Status::from_string(&status);
            self.loc = PathBuf::from(loc);
            self.dest_id = row.get("dest_id");
        };

        Ok(())
    }
}
