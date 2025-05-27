use anyhow::{Context, Result, anyhow};
use foundry_ui::UI;
use regex::Regex;
use semver::Version;
use shared::command::CommandExt;
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

impl Requirement<'_> {
    // TODO(#3404)
    fn validate_and_get_output(&self) -> (bool, String) {
        let version = self.get_version();
        let mut is_valid;

        let output = if let Ok(version) = version {
            is_valid = version >= self.minimal_version;
            let is_recommended = self
                .minimal_recommended_version
                .as_ref()
                .is_none_or(|minimal_recommended_version| version >= *minimal_recommended_version);

            let min_version_to_display = self
                .minimal_recommended_version
                .as_ref()
                .unwrap_or(&self.minimal_version);

            if !is_valid {
                is_valid = false;
                format!(
                    "❌ {} Version {} doesn't satisfy minimal {}\n{}",
                    self.name, version, min_version_to_display, self.helper_text
                )
            } else if !is_recommended {
                format!(
                    "⚠️  {} Version {} doesn't satisfy minimal recommended {}\n{}",
                    self.name,
                    version,
                    self.minimal_recommended_version.as_ref().unwrap(),
                    self.helper_text
                )
            } else {
                format!("✅ {} {}", self.name, version)
            }
        } else {
            is_valid = false;
            format!(
                "❌ {} is not installed or not added to the PATH\n{}\n",
                self.name, self.helper_text
            )
        };

        (is_valid, output)
    }

    fn get_version(&self) -> Result<Version> {
        let version_command_output = self.command.borrow_mut().output_checked()?;
        let raw_version = String::from_utf8_lossy(&version_command_output.stdout)
            .trim()
            .to_string();

        (self.version_parser)(&raw_version)
    }
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

    pub fn check(&self, ui: &UI) -> Result<()> {
        let (validation_output, all_requirements_valid) = self.check_and_prepare_output();

        if self.output_on_success || !all_requirements_valid {
            ui.println(&validation_output);
        }

        if all_requirements_valid {
            Ok(())
        } else {
            Err(anyhow!("Requirements not satisfied"))
        }
    }

    fn check_and_prepare_output(&self) -> (String, bool) {
        let mut validation_output = "Checking requirements\n\n".to_string();
        let mut all_valid = true;

        for requirement in &self.requirements {
            let (is_valid, output) = requirement.validate_and_get_output();

            all_valid &= is_valid;
            validation_output += output.as_str();
            validation_output += "\n";
        }

        (validation_output, all_valid)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::MINIMAL_SCARB_VERSION;
    use assert_fs::{
        TempDir,
        fixture::{FileWriteStr, PathChild},
    };
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
            minimal_version: MINIMAL_SCARB_VERSION,
            minimal_recommended_version: Some(Version::new(2, 9, 4)),
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

        let (validation_output, is_valid) = requirements_checker.check_and_prepare_output();

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

        let (validation_output, is_valid) = requirements_checker.check_and_prepare_output();
        assert!(!is_valid);
        assert!(validation_output.contains("❌ Rust Version"));
        assert!(validation_output.contains("doesn't satisfy minimal 999.0.0"));
    }

    #[test]
    #[cfg_attr(not(feature = "scarb_2_9_1"), ignore)]
    fn warning_requirements() {
        let mut requirements_checker = RequirementsChecker::new(true);
        requirements_checker.add_requirement(Requirement {
            name: "Scarb".to_string(),
            command: RefCell::new(ScarbCommand::new().arg("--version").command()),
            minimal_version: MINIMAL_SCARB_VERSION,
            minimal_recommended_version: Some(Version::new(999, 0, 0)),
            helper_text: "Follow instructions from https://docs.swmansion.com/scarb/download.html"
                .to_string(),
            version_parser: create_version_parser(
                "Scarb",
                r"scarb (?<version>[0-9]+.[0-9]+.[0-9]+)",
            ),
        });

        let (validation_output, is_valid) = requirements_checker.check_and_prepare_output();

        let ui = UI::default();
        ui.println(&validation_output);

        assert!(is_valid);
        assert!(validation_output.contains("⚠️  Scarb Version"));
        assert!(validation_output.contains("doesn't satisfy minimal recommended 999.0.0"));
    }

    #[test]
    fn failing_requirements_on_both_minimal_versions_defined() {
        let mut requirements_checker = RequirementsChecker::new(true);
        requirements_checker.add_requirement(Requirement {
            name: "Scarb".to_string(),
            command: RefCell::new(ScarbCommand::new().arg("--version").command()),
            minimal_version: Version::new(111, 0, 0),
            minimal_recommended_version: Some(Version::new(999, 0, 0)),
            helper_text: "Follow instructions from https://docs.swmansion.com/scarb/download.html"
                .to_string(),
            version_parser: create_version_parser(
                "Scarb",
                r"scarb (?<version>[0-9]+.[0-9]+.[0-9]+)",
            ),
        });

        let (validation_output, is_valid) = requirements_checker.check_and_prepare_output();

        assert!(!is_valid);
        assert!(validation_output.contains("❌ Scarb Version"));
        assert!(validation_output.contains("doesn't satisfy minimal 999.0.0"));
    }

    #[test]
    #[cfg_attr(not(feature = "no_scarb_installed"), ignore)]
    fn failing_tool_not_installed() {
        let temp_dir = TempDir::new().unwrap();
        temp_dir
            .child(".tool-versions")
            .write_str("scarb 2.9.9\n")
            .unwrap();

        let mut requirements_checker = RequirementsChecker::new(true);
        requirements_checker.add_requirement(Requirement {
            name: "Scarb".to_string(),
            command: RefCell::new(
                ScarbCommand::new()
                    .arg("--version")
                    .current_dir(temp_dir.path())
                    .command(),
            ),
            minimal_version: Version::new(2, 8, 5),
            minimal_recommended_version: Some(Version::new(2, 9, 4)),
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
            minimal_version: Version::new(2, 4, 0),
            minimal_recommended_version: None,
            helper_text: "Reinstall `snforge` using the same installation method or follow instructions from https://foundry-rs.github.io/starknet-foundry/getting-started/installation.html#universal-sierra-compiler-update".to_string(),
            version_parser: create_version_parser(
                "Universal Sierra Compiler",
                r"universal-sierra-compiler (?<version>[0-9]+.[0-9]+.[0-9]+)",
            ),
        });

        let (validation_output, is_valid) = requirements_checker.check_and_prepare_output();

        assert!(!is_valid);
        assert!(validation_output.contains("❌ Scarb is not installed or not added to the PATH"));
        assert!(
            validation_output.contains(
                "Follow instructions from https://docs.swmansion.com/scarb/download.html"
            )
        );
        assert!(validation_output.contains("✅ Universal Sierra Compiler"));
    }
}
