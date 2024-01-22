use crate::{
    output_transform::{OutputTransform, PassByPrint},
    ScarbCommand,
};
use anyhow::{Context, Result};
use regex::Regex;
use semver::Version;
use std::str::from_utf8;

pub struct ScarbVersionOutput {
    pub scarb: Version,
    pub cairo: Version,
}

pub struct ScarbVersionCommand<Stdout = PassByPrint, Stderr = PassByPrint> {
    inner: ScarbCommand<Stdout, Stderr>,
}

impl<Stdout, Stderr> ScarbVersionCommand<Stdout, Stderr> {
    pub(crate) fn new(inner: ScarbCommand<Stdout, Stderr>) -> Self {
        Self { inner }
    }

    /// Runs configured `scarb --version` and returns parsed `ScarbVersionOutput`.
    pub fn run(mut self) -> Result<ScarbVersionOutput>
    where
        Stdout: OutputTransform,
        Stderr: OutputTransform,
    {
        let scarb_version = self
            .inner
            .arg("--version")
            .run()
            .context("Failed to execute `scarb --version`")?;

        let version_output = from_utf8(&scarb_version.stdout)
            .context("Failed to parse `scarb --version` output to UTF-8")?;

        Ok(ScarbVersionOutput {
            scarb: Self::extract_version(version_output, "scarb")?,
            cairo: Self::extract_version(version_output, "cairo:")?,
        })
    }

    fn extract_version(version_output: &str, tool: &str) -> Result<Version> {
        let version_regex = Regex::new(&format!(r#"(?:{tool}?\s*)([0-9]+.[0-9]+.[0-9]+)"#))
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
