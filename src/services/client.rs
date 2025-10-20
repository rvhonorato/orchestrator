#[derive(Debug, thiserror::Error)]
pub enum ClientError {
    #[error("Execution error")]
    ExecutionError,
}
