use crate::ScarbCommand;
use anyhow::{Context, Result};
use regex::Regex;
use semver::Version;
use shared::command::CommandExt;
use std::path::PathBuf;
use std::str::from_utf8;

pub struct ScarbVersionOutput {
    pub scarb: Version,
    pub cairo: Version,
}

#[derive(Clone, Debug, Default)]
pub struct VersionCommand {
    current_dir: Option<PathBuf>,
}

impl VersionCommand {
    #[must_use]
    pub fn new() -> Self {
        Self { current_dir: None }
    }

    /// Current directory of the `scarb --command` process.
    #[must_use]
    pub fn current_dir(mut self, path: impl Into<PathBuf>) -> Self {
        self.current_dir = Some(path.into());
        self
    }
}

impl VersionCommand {
    pub fn run(self) -> Result<ScarbVersionOutput> {
        let mut scarb_version_command = ScarbCommand::new().arg("--version").command();

        if let Some(path) = &self.current_dir {
            scarb_version_command.current_dir(path);
        }

        let scarb_version = scarb_version_command
            .output_checked()
            .context("Failed to execute `scarb --version`")?;

        let version_output = from_utf8(&scarb_version.stdout)
            .context("Failed to parse `scarb --version` output to UTF-8")?;

        Ok(ScarbVersionOutput {
            scarb: Self::extract_version(version_output, "scarb")?,
            cairo: Self::extract_version(version_output, "cairo:")?,
        })
    }

    fn extract_version(version_output: &str, tool: &str) -> Result<Version> {
        // https://semver.org/#is-there-a-suggested-regular-expression-regex-to-check-a-semver-string
        let version_regex = Regex::new(
            &format!(r"{tool}?\s*((?:0|[1-9]\d*)\.(?:0|[1-9]\d*)\.(?:0|[1-9]\d*)(?:-((?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*)(?:\.(?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*))*))?(?:\+([0-9a-zA-Z-]+(?:\.[0-9a-zA-Z-]+)*))?)"))
            .context("Could not create version matching regex")?;

        let version_capture = version_regex
            .captures(version_output)
            .context(format!("Could not find {tool} version"))?;

        let version = version_capture
            .get(1)
            .context(format!("Could not find {tool} version"))?
            .as_str();

        Version::parse(version).context(format!("Failed to parse {tool} version"))
    }
}
