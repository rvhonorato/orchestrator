use serde::Serialize;

#[derive(Serialize)]
pub struct Ping {
    pub message: String,
}
