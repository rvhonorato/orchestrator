use sqlx::SqlitePool;

use crate::models::job_dao::Job;
use crate::models::status_dto::Status;

pub async fn create_jobs_table(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS jobs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id INTEGER NOT NULL,
            status TEXT NOT NULL,
            loc TEXT NOT NULL,
            dest_id TEXT NOT NULL,
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
        let result = sqlx::query("INSERT INTO jobs (user_id, loc, status) VALUES (?, ?, ?)")
            .bind(self.user_id)
            .bind(self.loc.to_str())
            .bind(self.status.to_string())
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
        let _result = sqlx::query("UPDATE jobs SET status = ? WHERE id = ?")
            .bind(&dest_id)
            .bind(self.id)
            .execute(pool)
            .await?;

        self.dest_id = dest_id;

        Ok(())
    }

    pub fn set_user_id(&mut self, user_id: i32) {
        self.user_id = user_id;
    }
}
