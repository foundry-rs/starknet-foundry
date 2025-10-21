//! Functionality for fetching and parsing `scarb` version information.

use crate::{ScarbCommand, ScarbUnavailableError, ensure_scarb_available};
use regex::Regex;
use semver::Version;
use shared::command::{CommandError, CommandExt};
use std::collections::HashMap;
use std::path::Path;
use std::process::Output;
use std::sync::LazyLock;
use thiserror::Error;

#[derive(Debug)]
pub struct ScarbVersionOutput {
    pub scarb: Version,
    pub cairo: Version,
    pub sierra: Version,
}

/// Errors that can occur when fetching `scarb` version information.
#[derive(Error, Debug)]
pub enum VersionError {
    #[error(transparent)]
    ScarbNotFound(#[from] ScarbUnavailableError),

    #[error("Failed to execute `scarb --version`: {0}")]
    CommandExecutionError(#[from] CommandError),

    #[error("Could not parse version for tool `{0}`: {1}")]
    VersionParseError(String, #[source] semver::Error),

    #[error("Missing version entry for `{0}`")]
    MissingToolVersion(String),
}

/// Fetches `scarb` version information for the current directory.
pub fn scarb_version() -> Result<ScarbVersionOutput, VersionError> {
    scarb_version_internal(None)
}

/// Fetches `scarb` version information for a specific directory.
pub fn scarb_version_for_dir(dir: impl AsRef<Path>) -> Result<ScarbVersionOutput, VersionError> {
    scarb_version_internal(Some(dir.as_ref()))
}

/// Internal function to fetch `scarb` version information.
fn scarb_version_internal(dir: Option<&Path>) -> Result<ScarbVersionOutput, VersionError> {
    let output = run_command(dir)?;
    parse_output(&output)
}

/// Runs the `scarb --version` command in the specified directory.
fn run_command(dir: Option<&Path>) -> Result<Output, VersionError> {
    ensure_scarb_available()?;
    let mut scarb_version_command = ScarbCommand::new().arg("--version").command();

    if let Some(path) = dir {
        scarb_version_command.current_dir(path);
    }

    Ok(scarb_version_command.output_checked()?)
}

/// Parses the output of the `scarb --version` command.
fn parse_output(output: &Output) -> Result<ScarbVersionOutput, VersionError> {
    let output_str = str::from_utf8(&output.stdout).expect("valid UTF-8 from scarb");

    let mut versions = extract_versions(output_str)?;

    let mut get_version = |tool: &str| {
        versions
            .remove(tool)
            .ok_or_else(|| VersionError::MissingToolVersion(tool.into()))
    };

    Ok(ScarbVersionOutput {
        scarb: get_version("scarb")?,
        cairo: get_version("cairo")?,
        sierra: get_version("sierra")?,
    })
}

/// Extracts tool versions from the version output string.
fn extract_versions(version_output: &str) -> Result<HashMap<String, Version>, VersionError> {
    // https://semver.org/#is-there-a-suggested-regular-expression-regex-to-check-a-semver-string
    static VERSION_REGEX: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"(?P<tool>\w+)[\s:]+(?P<ver>(?:0|[1-9]\d*)\.(?:0|[1-9]\d*)\.(?:0|[1-9]\d*)(?:-(?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*)(?:\.(?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*))*)?(?:\+[0-9a-zA-Z-]+(?:\.[0-9a-zA-Z-]+)*)?)").expect("this should be a valid regex")
    });

    VERSION_REGEX
        .captures_iter(version_output)
        .map(|cap| {
            let tool = cap
                .name("tool")
                .expect("regex ensures tool name exists")
                .as_str()
                .to_string();

            let ver_str = cap
                .name("ver")
                .expect("regex ensures version string exists")
                .as_str();

            Version::parse(ver_str)
                .map_err(|e| VersionError::VersionParseError(tool.clone(), e))
                .map(|version| (tool, version))
        })
        .collect()
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_happy_case() {
        scarb_version().unwrap();
    }
}
