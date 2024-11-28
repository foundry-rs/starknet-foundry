use anyhow::{anyhow, Context, Result};
use semver::Version;
use std::process::Command;

pub struct Requirement {
    pub name: String,
    pub command: String,
    pub minimal_version: Version,
}

pub struct RequirementsChecker {
    requirements: Vec<Requirement>,
}

impl RequirementsChecker {
    pub(crate) fn new() -> Self {
        Self {
            requirements: Vec::new(),
        }
    }

    pub fn add_requirement(&mut self, requirement: Requirement) {
        self.requirements.push(requirement);
    }

    pub fn validate(&self) -> Result<()> {
        let mut validation_output = "Validating requirements\n\n".to_string();
        let mut is_valid = true;

        for requirement in &self.requirements {
            let version = get_version(&requirement.name, &requirement.command)?;
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

fn get_version(name: &str, raw_command: &str) -> Result<Version> {
    let mut command = Command::new("sh");
    command.arg("-c").arg(raw_command);
    let raw_current_version = command
        .output()
        .with_context(|| format!("Failed to run version command for {name}"))?;
    let raw_current_version = String::from_utf8_lossy(&raw_current_version.stdout)
        .trim()
        .to_string();
    Version::parse(&raw_current_version).context("Failed to parse version")
}
