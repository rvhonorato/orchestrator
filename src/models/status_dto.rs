use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Status {
    Pending,
    Processing,
    Completed,
    Failed,
    Queued,
    Submitted,
    Unknown,
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Status::Pending => write!(f, "pending"),
            Status::Processing => write!(f, "processing"),
            Status::Completed => write!(f, "completed"),
            Status::Failed => write!(f, "failed"),
            Status::Queued => write!(f, "queued"),
            Status::Submitted => write!(f, "submitted"),
            Status::Unknown => write!(f, "unknown"),
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
            _ => Status::Unknown,
        }
    }
}
