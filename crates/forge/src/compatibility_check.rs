use anyhow::{anyhow, Context, Result};
use semver::Version;
use std::process::Command;

type VersionParser = dyn Fn(&str) -> Result<Version>;

pub struct Requirement {
    pub name: String,
    pub command: String,
    pub version_parser: Box<VersionParser>,
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
