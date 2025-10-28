use std::io;
use std::process::{Command, ExitStatus, Output};
use thiserror::Error;

/// Error type for command execution failures
#[derive(Debug, Error)]
pub enum CommandError {
    #[error("Failed to run {command}: {source}")]
    IoError {
        command: String,
        #[source]
        source: io::Error,
    },

    #[error("Command {command} failed with status {status}")]
    FailedStatus { command: String, status: ExitStatus },
}

/// Trait extension for Command to check output status
pub trait CommandExt {
    fn output_checked(&mut self) -> Result<Output, CommandError>;
}

impl CommandExt for Command {
    fn output_checked(&mut self) -> Result<Output, CommandError> {
        let command = self.get_program().to_string_lossy().to_string();

        match self.output() {
            Ok(output) => {
                if output.status.success() {
                    Ok(output)
                } else {
                    Err(CommandError::FailedStatus {
                        command,
                        status: output.status,
                    })
                }
            }
            Err(source) => Err(CommandError::IoError { command, source }),
        }
    }
}
