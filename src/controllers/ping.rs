use crate::models::ping_dto::Ping;
use axum::extract::Json;

pub async fn ping() -> Json<Ping> {
    Json(Ping {
        message: "pong".to_string(),
    })
}
