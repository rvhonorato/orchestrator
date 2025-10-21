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

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_execute_payload() {
        // Prepare a temporary payload
        let temp_dir = tempfile::tempdir().unwrap();
        let mut payload = Payload::new();
        payload.set_loc(temp_dir.path().to_path_buf());

        // Add a simple run.sh script
        std::fs::write(payload.loc.join("run.sh"), b"#!/bin/bash").unwrap();

        let result = execute_payload(&payload);

        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_payload_no_script() {
        // Prepare a temporary payload
        let temp_dir = tempfile::tempdir().unwrap();
        let mut payload = Payload::new();
        payload.set_loc(temp_dir.path().to_path_buf());

        let result = execute_payload(&payload);

        assert!(matches!(result, Err(ClientError::NoExecScript)));
    }

    #[test]
    fn test_execute_payload_script_error() {
        // Prepare a temporary payload
        let temp_dir = tempfile::tempdir().unwrap();
        let mut payload = Payload::new();
        payload.set_loc(temp_dir.path().to_path_buf());

        // Add a run.sh script that fails
        std::fs::write(payload.loc.join("run.sh"), b"#!/bin/bash\nexit 1").unwrap();

        let result = execute_payload(&payload);

        assert!(matches!(result, Err(ClientError::Script)));
    }
}
