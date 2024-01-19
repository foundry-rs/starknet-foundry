use crate::ScarbCommand;
use anyhow::Context;
use regex::Regex;
use semver::Version;
use std::str::from_utf8;

pub struct ScarbVersionOutput {
    pub scarb: Version,
    pub cairo: Version,
}

#[must_use]
pub fn scarb_version_command() -> ScarbVersionOutput {
    let scarb_version = ScarbCommand::new()
        .arg("--version")
        .command()
        .output()
        .context("Failed to execute `scarb --version`")
        .unwrap();
    let version_output = from_utf8(&scarb_version.stdout)
        .context("Failed to parse `scarb --version` output to UTF-8")
        .unwrap();

    ScarbVersionOutput {
        scarb: extract_version(version_output, "scarb"),
        cairo: extract_version(version_output, "cairo"),
    }
}

fn extract_version(version_output: &str, tool: &str) -> Version {
    let version_regex = Regex::new(&format!(r#"(?:{tool}?\s*)([0-9]+.[0-9]+.[0-9]+)"#))
        .expect("Could not create version matching regex");
    let version_capture = version_regex
        .captures(version_output)
        .unwrap_or_else(|| panic!("Could not find {tool} version"));
    let scarb_version = version_capture
        .get(1)
        .unwrap_or_else(|| panic!("Could not find {tool} version"))
        .as_str();
    Version::parse(scarb_version).unwrap_or_else(|_| panic!("Failed to parse {tool} version"))
}
