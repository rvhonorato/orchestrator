use std::process::Command;

use crate::models::payload_dao::Payload;
use tracing::info;

#[derive(Debug, thiserror::Error)]
pub enum ClientError {
    #[error("Execution error")]
    Execution,
    #[error("Script error")]
    Script,
    #[error("No execution script found")]
    NoExecScript,
}

pub fn execute_payload(payload: &Payload) -> Result<(), ClientError> {
    info!("{:?}", payload);

    // Expect the payload.loc to contain a `run.sh` script
    let run_script = payload.loc.join("run.sh");

    // Make sure the script exists
    if !run_script.exists() {
        return Err(ClientError::NoExecScript);
    }

    // Execute script and wait for it to finish
    let exit_status = Command::new("bash")
        .arg(run_script)
        .current_dir(&payload.loc)
        .status()
        .map_err(|_| ClientError::Execution)?;

    if !exit_status.success() {
        return Err(ClientError::Script);
    }

    Ok(())
}
