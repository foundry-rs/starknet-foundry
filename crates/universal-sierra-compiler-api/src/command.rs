use shared::command::{CommandError, CommandExt};
use std::process::Output;
use std::{
    env,
    ffi::OsStr,
    process::{Command, Stdio},
};
use thiserror::Error;

/// Errors that can occur while working with `universal-sierra-compiler` command.
#[derive(Debug, Error)]
pub enum USCError {
    #[error(
        "`universal-sierra-compiler` binary not available. \
             Make sure it is installed and available in PATH or set via UNIVERSAL_SIERRA_COMPILER."
    )]
    NotFound(#[source] which::Error),

    #[error(
        "Error while compiling Sierra. \
             Make sure you have the latest universal-sierra-compiler binary installed. \
             Contact us if it doesn't help."
    )]
    RunFailed(#[source] CommandError),
}

/// An internal builder for `universal-sierra-compiler` command invocation.
#[derive(Debug)]
pub struct USCInternalCommand {
    inner: Command,
}

impl USCInternalCommand {
    /// Creates a new `universal-sierra-compiler` command builder.
    pub fn new() -> Result<Self, USCError> {
        ensure_available()?;
        let mut cmd = Command::new(binary_path());
        cmd.stderr(Stdio::inherit());
        Ok(Self { inner: cmd })
    }

    /// Adds an argument to pass to `universal-sierra-compiler`.
    pub fn arg(mut self, arg: impl AsRef<OsStr>) -> Self {
        self.inner.arg(arg);
        self
    }

    /// Returns the constructed [`Command`].
    #[must_use]
    pub fn command(self) -> Command {
        self.inner
    }

    /// Runs the `universal-sierra-compiler` command and returns the [`Output`].
    pub fn run(self) -> Result<Output, USCError> {
        self.command().output_checked().map_err(USCError::RunFailed)
    }
}

/// Ensures that `universal-sierra-compiler` binary is available in the system.
pub fn ensure_available() -> Result<(), USCError> {
    which::which(binary_path())
        .map(|_| ())
        .map_err(USCError::NotFound)
}

/// Returns the binary path either from env or fallback to default name.
fn binary_path() -> String {
    env::var("UNIVERSAL_SIERRA_COMPILER")
        .unwrap_or_else(|_| "universal-sierra-compiler".to_string())
}
