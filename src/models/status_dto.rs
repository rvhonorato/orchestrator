use serde::{Deserialize, Serialize};
use std::fmt;
use utoipa::ToSchema;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub enum Status {
    Pending,
    Processing,
    Completed,
    Failed,
    Queued,
    Submitted,
    Unknown,
    Cleaned,
    Prepared,
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Status::Pending => write!(f, "pending"),
            Status::Prepared => write!(f, "prepared"),
            Status::Processing => write!(f, "processing"),
            Status::Completed => write!(f, "completed"),
            Status::Failed => write!(f, "failed"),
            Status::Queued => write!(f, "queued"),
            Status::Submitted => write!(f, "submitted"),
            Status::Unknown => write!(f, "unknown"),
            Status::Cleaned => write!(f, "cleaned"),
        }
    }
}

impl Status {
    pub fn from_string(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "pending" => Status::Pending,
            "processing" => Status::Processing,
            "completed" => Status::Completed,
            "failed" => Status::Failed,
            "queued" => Status::Queued,
            "submitted" => Status::Submitted,
            "cleaned" => Status::Cleaned,
            _ => Status::Unknown,
        }
    }
}
