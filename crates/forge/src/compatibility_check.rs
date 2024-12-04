use anyhow::{anyhow, Context, Result};
use regex::Regex;
use semver::Version;
use std::process::Command;

type VersionParser<'a> = dyn Fn(&str) -> Result<Version> + 'a;

pub struct Requirement<'a> {
    pub name: String,
    pub command: String,
    pub version_parser: Box<VersionParser<'a>>,
    pub minimal_version: Version,
}

pub struct RequirementsChecker<'a> {
    requirements: Vec<Requirement<'a>>,
}

impl<'a> RequirementsChecker<'a> {
    pub(crate) fn new() -> Self {
        Self {
            requirements: Vec::new(),
        }
    }

    pub fn add_requirement(&mut self, requirement: Requirement<'a>) {
        self.requirements.push(requirement);
    }

    pub fn validate(&self) -> Result<()> {
        let mut validation_output = "Validating requirements\n\n".to_string();
        let mut is_valid = true;

        for requirement in &self.requirements {
            let raw_version = get_raw_version(&requirement.name, &requirement.command)?;
            let version = (requirement.version_parser)(&raw_version)?;
            let valid = version >= requirement.minimal_version;
            let output = if valid {
                format!("✅ {} {}", requirement.name, version)
            } else {
                is_valid = false;
                format!(
                    "❌ {} Version {} doesn't satisfy minimum {}",
                    requirement.name, version, requirement.minimal_version
                )
            };

            validation_output += output.as_str();
            validation_output += "\n";
        }

        if !is_valid {
            println!("{validation_output}");
            return Err(anyhow!("Requirements not satisfied"));
        }

        Ok(())
    }
}

pub fn create_version_parser<'a>(name: &'a str, pattern: &'a str) -> Box<VersionParser<'a>> {
    let regex = Regex::new(pattern).unwrap();
    Box::new(move |raw_version: &str| {
        let matches = regex.captures(raw_version).with_context(|| {
            format!("Failed to match {name} version from output: {raw_version}",)
        })?;
        let version_str = matches
            .name("version")
            .with_context(|| format!("Failed to parse {name} version"))?
            .as_str();
        Version::parse(version_str).with_context(|| "Failed to parse version")
    })
}

fn os_specific_command() -> Command {
    if cfg!(target_os = "windows") {
        let mut command = Command::new("cmd");
        command.arg("/C");
        command
    } else {
        let mut command = Command::new("sh");
        command.arg("-c");
        command
    }
}

fn get_raw_version(name: &str, raw_command: &str) -> Result<String> {
    let mut command = os_specific_command();
    command.arg(raw_command);

    let raw_current_version = command
        .output()
        .with_context(|| format!("Failed to run version command for {name}"))?;
    Ok(String::from_utf8_lossy(&raw_current_version.stdout)
        .trim()
        .to_string())
}
