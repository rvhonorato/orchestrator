use crate::models::ping_dto::Ping;
use axum::extract::Json;
use utoipa;

#[utoipa::path(get, path = "/")]
pub async fn ping() -> Json<Ping> {
    Json(Ping {
        message: "pong".to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_ping_returns_pong() {
        let response = ping().await;
        let ping_response = response.0;
        assert_eq!(ping_response.message, "pong");
    }
}
