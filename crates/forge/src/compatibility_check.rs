use crate::MINIMAL_SIERRA_VERSION_FOR_SIERRA_GAS;
use anyhow::{Context, Result, anyhow, bail};
use forge_runner::forge_config::{ForgeConfig, ForgeTrackedResource};
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

            let is_valid = version >= requirement.minimal_version;
            let is_recommended = requirement
                .minimal_recommended_version
                .as_ref()
                .is_none_or(|minimal_recommended_version| version >= *minimal_recommended_version);

            let min_version_to_display = requirement
                .minimal_recommended_version
                .as_ref()
                .unwrap_or(&requirement.minimal_version);

            let command_output = if !is_valid {
                all_valid = false;
                format!(
                    "❌ {} Version {} doesn't satisfy minimal {}\n{}",
                    requirement.name, version, min_version_to_display, requirement.helper_text
                )
            } else if !is_recommended {
                format!(
                    "⚠️  {} Version {} doesn't satisfy minimal recommended {}\n{}",
                    requirement.name,
                    version,
                    requirement.minimal_recommended_version.as_ref().unwrap(),
                    requirement.helper_text
                )
            } else {
                format!("✅ {} {}", requirement.name, version)
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

fn verify_contracts_sierra_versions(forge_config: &ForgeConfig) -> Result<()> {
    if let Some(contract_names) = forge_config
        .test_runner_config
        .contracts_data
        .get_contract_names()
    {
        for contract in contract_names {
            let sierra_str = &forge_config
                .test_runner_config
                .contracts_data
                .get_artifacts(contract)
                .context(format!("Missing Sierra JSON for contract: {contract}"))?
                .sierra;

            let contract_version = parse_sierra_version(sierra_str)?;
            if MINIMAL_SIERRA_VERSION_FOR_SIERRA_GAS > contract_version {
                bail!(
                    "Contract version {} is lower than required minimal sierra version",
                    contract_version
                );
            }
        }
    }
    Ok(())
}

fn parse_sierra_version(sierra_str: &str) -> Result<Version> {
    let sierra_json: serde_json::Value =
        serde_json::from_str(sierra_str).context("Failed to parse Sierra JSON")?;

    let parsed_values: Vec<u8> = sierra_json["sierra_program"]
        .as_array()
        .context("Unable to read `sierra_program`. Ensure it is an array of felts.")?
        .iter()
        .take(3)
        .filter_map(|x| x.as_str())
        .map(|s| {
            u8::from_str_radix(s.strip_prefix("0x").unwrap_or(s), 16)
                .context(format!("Invalid hex value: {s}"))
        })
        .collect::<Result<_, _>>()?;

    Version::parse(
        format!(
            "{}.{}.{}",
            &parsed_values[0], &parsed_values[1], &parsed_values[2]
        )
        .as_str(),
    )
    .map_err(std::convert::Into::into)
}

pub(crate) fn check_sierra_gas_version_requirement(forge_config: &ForgeConfig) -> Result<()> {
    if forge_config.test_runner_config.tracked_resource == ForgeTrackedResource::SierraGas {
        verify_contracts_sierra_versions(forge_config).with_context(||
            format!(
                "Tracking SierraGas is not supported for sierra <= {MINIMAL_SIERRA_VERSION_FOR_SIERRA_GAS}"
            ))?;
    };
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use camino::Utf8PathBuf;
    use cheatnet::runtime_extensions::forge_runtime_extension::contracts_data::{
        ContractName, ContractsData,
    };
    use forge_runner::forge_config::{ExecutionDataToSave, OutputConfig, TestRunnerConfig};
    use scarb_api::{ScarbCommand, StarknetContractArtifacts};
    use std::collections::HashMap;
    use std::num::NonZeroU32;
    use std::sync::Arc;
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
            minimal_recommended_version: Some(Version::new(2, 8, 5)),
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
        assert!(validation_output.contains("doesn't satisfy minimal 999.0.0"));
    }

    #[test]
    #[cfg_attr(not(feature = "scarb_2_7_1"), ignore)]
    fn warning_requirements() {
        let mut requirements_checker = RequirementsChecker::new(true);
        requirements_checker.add_requirement(Requirement {
            name: "Scarb".to_string(),
            command: RefCell::new(ScarbCommand::new().arg("--version").command()),
            minimal_version: Version::new(2, 7, 0),
            minimal_recommended_version: Some(Version::new(999, 0, 0)),
            helper_text: "Follow instructions from https://docs.swmansion.com/scarb/download.html"
                .to_string(),
            version_parser: create_version_parser(
                "Scarb",
                r"scarb (?<version>[0-9]+.[0-9]+.[0-9]+)",
            ),
        });

        let (validation_output, is_valid) =
            requirements_checker.check_and_prepare_output().unwrap();

        println!("{validation_output}");
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

        let (validation_output, is_valid) =
            requirements_checker.check_and_prepare_output().unwrap();

        assert!(!is_valid);
        assert!(validation_output.contains("❌ Scarb Version"));
        assert!(validation_output.contains("doesn't satisfy minimal 999.0.0"));
    }

    #[test]
    fn sierra_gas_version_requirement_happy_case() {
        let mut contracts = HashMap::new();
        contracts.insert(ContractName::from("MockedContract"), 
                         (StarknetContractArtifacts {
                             sierra: String::from("{\"sierra_program\":[\"0x1\",\"0x7\",\"0x0\"],\"sierra_program_debug_info\":{\"type_names\":[],\"libfunc_names\":[],\"user_func_names\":[]},\"contract_class_version\":\"0.1.0\",\"entry_points_by_type\":{\"EXTERNAL\":[],\"L1_HANDLER\":[],\"CONSTRUCTOR\":[]},\"abi\":[]}"),
                             casm: String::new()},
                          Utf8PathBuf::from("whatever")
                         ));

        let forge_config = ForgeConfig {
            test_runner_config: Arc::new(TestRunnerConfig {
                exit_first: false,
                fuzzer_runs: NonZeroU32::new(256).unwrap(),
                fuzzer_seed: 12345,
                max_n_steps: None,
                is_vm_trace_needed: false,
                tracked_resource: ForgeTrackedResource::SierraGas,
                cache_dir: Utf8PathBuf::default(),
                contracts_data: ContractsData::try_from(contracts).unwrap(),
                environment_variables: HashMap::default(),
            }),
            output_config: Arc::new(OutputConfig {
                detailed_resources: false,
                execution_data_to_save: ExecutionDataToSave::default(),
            }),
        };
        assert!(check_sierra_gas_version_requirement(&forge_config).is_ok());
    }

    #[test]
    fn sierra_gas_version_requirement_cairo_steps() {
        let mut contracts = HashMap::new();
        contracts.insert(ContractName::from("MockedContract"),
                         (StarknetContractArtifacts {
                             sierra: String::from("{\"sierra_program\":[\"0x1\",\"0x6\",\"0x0\"],\"sierra_program_debug_info\":{\"type_names\":[],\"libfunc_names\":[],\"user_func_names\":[]},\"contract_class_version\":\"0.1.0\",\"entry_points_by_type\":{\"EXTERNAL\":[],\"L1_HANDLER\":[],\"CONSTRUCTOR\":[]},\"abi\":[]}"),
                             casm: String::new()},
                          Utf8PathBuf::from("whatever")
                         ));

        let forge_config = ForgeConfig {
            test_runner_config: Arc::new(TestRunnerConfig {
                exit_first: false,
                fuzzer_runs: NonZeroU32::new(256).unwrap(),
                fuzzer_seed: 12345,
                max_n_steps: None,
                is_vm_trace_needed: false,
                tracked_resource: ForgeTrackedResource::CairoSteps,
                cache_dir: Utf8PathBuf::default(),
                contracts_data: ContractsData::try_from(contracts).unwrap(),
                environment_variables: HashMap::default(),
            }),
            output_config: Arc::new(OutputConfig {
                detailed_resources: false,
                execution_data_to_save: ExecutionDataToSave::default(),
            }),
        };
        assert!(check_sierra_gas_version_requirement(&forge_config).is_ok());
    }

    #[test]
    fn sierra_gas_version_requirement_no_contracts() {
        let forge_config = ForgeConfig {
            test_runner_config: Arc::new(TestRunnerConfig {
                exit_first: false,
                fuzzer_runs: NonZeroU32::new(256).unwrap(),
                fuzzer_seed: 12345,
                max_n_steps: None,
                is_vm_trace_needed: false,
                tracked_resource: ForgeTrackedResource::SierraGas,
                cache_dir: Utf8PathBuf::default(),
                contracts_data: ContractsData::default(),
                environment_variables: HashMap::default(),
            }),
            output_config: Arc::new(OutputConfig {
                detailed_resources: false,
                execution_data_to_save: ExecutionDataToSave::default(),
            }),
        };
        assert!(check_sierra_gas_version_requirement(&forge_config).is_ok());
    }

    #[test]
    fn sierra_gas_version_requirement_fail() {
        let mut contracts = HashMap::new();
        contracts.insert(ContractName::from("MockedContract"),
                         (StarknetContractArtifacts {
                             sierra: String::from("{\"sierra_program\":[\"0x1\",\"0x6\",\"0x0\"],\"sierra_program_debug_info\":{\"type_names\":[],\"libfunc_names\":[],\"user_func_names\":[]},\"contract_class_version\":\"0.1.0\",\"entry_points_by_type\":{\"EXTERNAL\":[],\"L1_HANDLER\":[],\"CONSTRUCTOR\":[]},\"abi\":[]}"),
                             casm: String::new()},
                          Utf8PathBuf::from("whatever")
                         ));

        let forge_config = ForgeConfig {
            test_runner_config: Arc::new(TestRunnerConfig {
                exit_first: false,
                fuzzer_runs: NonZeroU32::new(256).unwrap(),
                fuzzer_seed: 12345,
                max_n_steps: None,
                is_vm_trace_needed: false,
                tracked_resource: ForgeTrackedResource::SierraGas,
                cache_dir: Utf8PathBuf::default(),
                contracts_data: ContractsData::try_from(contracts).unwrap(),
                environment_variables: HashMap::default(),
            }),
            output_config: Arc::new(OutputConfig {
                detailed_resources: false,
                execution_data_to_save: ExecutionDataToSave::default(),
            }),
        };
        assert!(check_sierra_gas_version_requirement(&forge_config).is_err());
    }
}
