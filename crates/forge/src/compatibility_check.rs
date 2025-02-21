use anyhow::{anyhow, Context, Result};
use regex::Regex;
use semver::Version;
use std::cell::RefCell;
use std::process::Command;

type VersionParser<'a> = dyn Fn(&str) -> Result<Version> + 'a;

pub struct Requirement<'a> {
    pub name: String,
    pub command: RefCell<Command>,
    pub version_parser: Box<VersionParser<'a>>,
    pub helper_text: String,
    pub minimal_version: Version,
    pub minimal_recommended_version: Option<Version>,
}

pub struct RequirementsChecker<'a> {
    output_on_success: bool,
    requirements: Vec<Requirement<'a>>,
}

impl<'a> RequirementsChecker<'a> {
    pub(crate) fn new(output_on_success: bool) -> Self {
        Self {
            output_on_success,
            requirements: Vec::new(),
        }
    }

    pub fn add_requirement(&mut self, requirement: Requirement<'a>) {
        self.requirements.push(requirement);
    }

    pub fn check(&self) -> Result<()> {
        let (validation_output, all_requirements_valid) = self.check_and_prepare_output()?;

        if self.output_on_success || !all_requirements_valid {
            println!("{validation_output}");
        }

        if all_requirements_valid {
            Ok(())
        } else {
            Err(anyhow!("Requirements not satisfied"))
        }
    }

    fn check_and_prepare_output(&self) -> Result<(String, bool)> {
        let mut validation_output = "Checking requirements\n\n".to_string();
        let mut all_valid = true;

        for requirement in &self.requirements {
            let raw_version = get_raw_version(&requirement.name, &requirement.command)?;
            let version = (requirement.version_parser)(&raw_version)?;
            let is_valid = if let Some(minimal_recommended_version) =
                &requirement.minimal_recommended_version
            {
                minimal_recommended_version <= &version
            } else {
                requirement.minimal_version <= version
            };
            let command_output = match (is_valid, &requirement.minimal_recommended_version) {
                (true, _) => format!("✅ {} {}", requirement.name, version),
                (false, Some(minimal_recommended_version)) => format!(
                    "⚠️  {} Version {} doesn't satisfy minimum recommended {}\n{}",
                    requirement.name, version, minimal_recommended_version, requirement.helper_text
                ),
                (false, None) => {
                    all_valid = false;
                    format!(
                        "❌ {} Version {} doesn't satisfy minimum {}\n{}",
                        requirement.name,
                        version,
                        requirement.minimal_version,
                        requirement.helper_text
                    )
                }
            };

            validation_output += command_output.as_str();
            validation_output += "\n";
        }

        Ok((validation_output, all_valid))
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

fn get_raw_version(name: &str, command: &RefCell<Command>) -> Result<String> {
    let raw_current_version = command
        .borrow_mut()
        .output()
        .with_context(|| format!("Failed to run version command for {name}"))?;
    Ok(String::from_utf8_lossy(&raw_current_version.stdout)
        .trim()
        .to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use scarb_api::ScarbCommand;
    use universal_sierra_compiler_api::UniversalSierraCompilerCommand;

    #[test]
    fn happy_case() {
        let mut requirements_checker = RequirementsChecker::new(true);
        requirements_checker.add_requirement(Requirement {
            name: "Rust".to_string(),
            command: RefCell::new({
                let mut cmd = Command::new("rustc");
                cmd.arg("--version");
                cmd
            }),
            version_parser: create_version_parser(
                "Rust",
                r"rustc (?<version>[0-9]+.[0-9]+.[0-9]+)",
            ),
            helper_text: "Follow instructions from https://www.rust-lang.org/tools/install"
                .to_string(),
            minimal_version: Version::new(1, 80, 1),
            minimal_recommended_version: None,
        });
        requirements_checker.add_requirement(Requirement {
            name: "Scarb".to_string(),
            command: RefCell::new(ScarbCommand::new().arg("--version").command()),
            minimal_version: Version::new(2, 7, 0),
            minimal_recommended_version: None,
            helper_text: "Follow instructions from https://docs.swmansion.com/scarb/download.html"
                .to_string(),
            version_parser: create_version_parser(
                "Scarb",
                r"scarb (?<version>[0-9]+.[0-9]+.[0-9]+)",
            ),
        });
        requirements_checker.add_requirement(Requirement {
            name: "Universal Sierra Compiler".to_string(),
            command: RefCell::new(UniversalSierraCompilerCommand::new().arg("--version").command()),
            minimal_version: Version::new(2, 0, 0),
             minimal_recommended_version: None,
            helper_text: "Reinstall `snforge` using the same installation method or follow instructions from https://foundry-rs.github.io/starknet-foundry/getting-started/installation.html#universal-sierra-compiler-update".to_string(),
            version_parser: create_version_parser(
                "Universal Sierra Compiler",
                r"universal-sierra-compiler (?<version>[0-9]+.[0-9]+.[0-9]+)",
            ),
        });

        let (validation_output, is_valid) =
            requirements_checker.check_and_prepare_output().unwrap();
        assert!(is_valid);
        assert!(validation_output.contains("✅ Rust"));
        assert!(validation_output.contains("✅ Scarb"));
        assert!(validation_output.contains("✅ Universal Sierra Compiler"));
    }

    #[test]
    fn failing_requirements() {
        let mut requirements_checker = RequirementsChecker::new(true);
        requirements_checker.add_requirement(Requirement {
            name: "Rust".to_string(),
            command: RefCell::new({
                let mut cmd = Command::new("rustc");
                cmd.arg("--version");
                cmd
            }),
            version_parser: create_version_parser(
                "Rust",
                r"rustc (?<version>[0-9]+.[0-9]+.[0-9]+)",
            ),
            helper_text: "Follow instructions from https://www.rust-lang.org/tools/install"
                .to_string(),
            minimal_version: Version::new(999, 0, 0),
            minimal_recommended_version: None,
        });

        let (validation_output, is_valid) =
            requirements_checker.check_and_prepare_output().unwrap();
        assert!(!is_valid);
        assert!(validation_output.contains("❌ Rust Version"));
        assert!(validation_output.contains("doesn't satisfy minimum 999.0.0"));
    }
}
