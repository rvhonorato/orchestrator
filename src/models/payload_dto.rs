use crate::models::payload_dao::Payload;
use crate::models::status_dto::Status;
use sqlx::SqlitePool;

pub async fn create_payload_table(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS payloads (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            status TEXT NOT NULL,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )
    "#,
    )
    .execute(pool)
    .await?;
    Ok(())
}

impl Payload {
    pub async fn add_to_db(&mut self, pool: &SqlitePool) -> Result<(), sqlx::Error> {
        let result = sqlx::query("INSERT INTO payloads (status) VALUES (?)")
            .bind(self.status.to_string())
            .execute(pool)
            .await?;

        let job_id = result.last_insert_rowid();
        self.id = job_id as u32;

        Ok(())
    }

    pub async fn update_status(
        &mut self,
        status: Status,
        pool: &SqlitePool,
    ) -> Result<(), sqlx::Error> {
        let _result = sqlx::query("UPDATE payloads SET status = ? WHERE id = ?")
            .bind(status.to_string())
            .bind(self.id)
            .execute(pool)
            .await?;

        self.status = status;

        Ok(())
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[tokio::test]
    async fn test_add_to_db() {
        let pool = crate::datasource::db::init_payload_db().await;

        let mut payload = Payload::new();

        let result = payload.add_to_db(&pool).await;
        assert!(result.is_ok());
        assert!(payload.id > 0);
    }

    #[tokio::test]
    async fn test_update_status() {
        let pool = crate::datasource::db::init_payload_db().await;

        let mut payload = Payload::new();

        payload
            .add_to_db(&pool)
            .await
            .expect("Failed to add payload to DB");

        assert_eq!(payload.status, Status::Unknown);

        payload
            .update_status(Status::Prepared, &pool)
            .await
            .expect("Failed to update payload status");

        assert_eq!(payload.status, Status::Prepared);
    }
}
