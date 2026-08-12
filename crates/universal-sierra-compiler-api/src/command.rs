use semver::Version;
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
          Make sure it is installed https://github.com/software-mansion/universal-sierra-compiler \
          and available in PATH or set via UNIVERSAL_SIERRA_COMPILER."
    )]
    NotFound(#[source] which::Error),

    #[error(
        "Error while compiling Sierra. \
         Make sure you have the latest universal-sierra-compiler binary installed. \
         Contact Starknet Foundry team through Github or Telegram if it doesn't help."
    )]
    RunFailed(#[source] CommandError),

    #[error("Failed to parse universal-sierra-compiler version from output: {0}")]
    VersionParseFailed(String),
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

/// Returns whether the current `universal-sierra-compiler` binary supports `--cache-dir`.
pub fn supports_cache_dir() -> Result<bool, USCError> {
    let current_version_supports_cache =
        current_version().map(|version| version_supports_cache(&version))?;

    Ok(current_version_supports_cache)
}

fn current_version() -> Result<Version, USCError> {
    let output = USCInternalCommand::new()?.arg("--version").run()?;
    let raw_version = String::from_utf8_lossy(&output.stdout);

    parse_version(&raw_version)
}

fn parse_version(raw_version: &str) -> Result<Version, USCError> {
    raw_version
        .split_whitespace()
        .find_map(|part| Version::parse(part).ok())
        .ok_or_else(|| USCError::VersionParseFailed(raw_version.to_string()))
}

fn version_supports_cache(version: &Version) -> bool {
    // TODO: Once USC releases version with cache support, ensure the version here is correct.
    version >= &Version::new(2, 9, 2)
}

/// Returns the binary path either from env or fallback to default name.
fn binary_path() -> String {
    env::var("UNIVERSAL_SIERRA_COMPILER")
        .unwrap_or_else(|_| "universal-sierra-compiler".to_string())
}

#[cfg(test)]
mod tests {
    use super::{parse_version, version_supports_cache};
    use semver::Version;

    #[test]
    fn parses_version_from_usc_output() {
        let version =
            parse_version("universal-sierra-compiler 2.9.2\n").expect("version should parse");

        assert_eq!(version, Version::new(2, 9, 2));
    }

    #[test]
    fn cache_dir_support_starts_at_2_9_2() {
        assert!(!version_supports_cache(&Version::new(2, 9, 1)));
        assert!(version_supports_cache(&Version::new(2, 9, 2)));
        assert!(version_supports_cache(&Version::new(2, 10, 0)));
    }
}
