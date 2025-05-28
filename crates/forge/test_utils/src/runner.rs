use crate::{get_assert_macros_version, tempdir_with_tool_versions};
use anyhow::{Context, Result, anyhow};
use assert_fs::{
    TempDir,
    fixture::{FileTouch, FileWriteStr, PathChild},
};
use blockifier::execution::{
    deprecated_syscalls::DeprecatedSyscallSelector, syscalls::vm_syscall_utils::SyscallUsage,
};
use cairo_vm::types::builtin_name::BuiltinName;
use camino::Utf8PathBuf;
use forge_runner::{
    test_case_summary::{AnyTestCaseSummary, TestCaseSummary},
    test_target_summary::TestTargetSummary,
};
use indoc::formatdoc;
use scarb_api::{
    ScarbCommand, StarknetContractArtifacts, get_contracts_artifacts_and_source_sierra_paths,
    metadata::MetadataCommandExt, target_dir_for_workspace,
};
use shared::command::CommandExt;
use starknet_api::execution_resources::{GasAmount, GasVector};
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    str::FromStr,
};

/// Represents a dependency of a Cairo project
#[derive(Debug, Clone)]
pub struct LinkedLibrary {
    pub name: String,
    pub path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct Contract {
    name: String,
    code: String,
}

impl Contract {
    #[must_use]
    pub fn new(name: impl Into<String>, code: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            code: code.into(),
        }
    }

    pub fn from_code_path(name: impl Into<String>, path: &Path) -> Result<Self> {
        let code = fs::read_to_string(path)?;
        Ok(Self {
            name: name.into(),
            code,
        })
    }

    fn generate_sierra_and_casm(self) -> Result<(String, String)> {
        let dir = tempdir_with_tool_versions()?;

        let contract_path = dir.child("src/lib.cairo");
        contract_path.touch()?;
        contract_path.write_str(&self.code)?;

        let scarb_toml_path = dir.child("Scarb.toml");
        scarb_toml_path
            .write_str(&formatdoc!(
                r#"
                [package]
                name = "contract"
                version = "0.1.0"

                [[target.starknet-contract]]
                sierra = true

                [dependencies]
                starknet = "2.6.4"
                "#,
            ))
            .unwrap();

        Command::new("scarb")
            .current_dir(&dir)
            .arg("build")
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .output_checked()
            .context("Failed to build contracts with Scarb")?;

        let scarb_metadata = ScarbCommand::metadata()
            .current_dir(dir.path())
            .inherit_stderr()
            .run()?;
        let package = scarb_metadata
            .packages
            .iter()
            .find(|package| package.name == "contract")
            .unwrap();
        let artifacts_dir = target_dir_for_workspace(&scarb_metadata).join("dev");

        let contract =
            get_contracts_artifacts_and_source_sierra_paths(&artifacts_dir, package, false)
                .unwrap()
                .remove(&self.name)
                .ok_or(anyhow!("there is no contract with name {}", self.name))?
                .0;

        Ok((contract.sierra, contract.casm))
    }
}

#[derive(Debug)]
pub struct TestCase {
    dir: TempDir,
    contracts: Vec<Contract>,
    environment_variables: HashMap<String, String>,
}

impl<'a> TestCase {
    pub const TEST_PATH: &'a str = "tests/test_case.cairo";
    const PACKAGE_NAME: &'a str = "my_package";

    pub fn from(test_code: &str, contracts: Vec<Contract>) -> Result<Self> {
        let dir = tempdir_with_tool_versions()?;
        let test_file = dir.child(Self::TEST_PATH);
        test_file.touch()?;
        test_file.write_str(test_code)?;

        dir.child("src/lib.cairo").touch().unwrap();

        let snforge_std_path = Utf8PathBuf::from_str("../../snforge_std")
            .unwrap()
            .canonicalize_utf8()
            .unwrap()
            .to_string()
            .replace('\\', "/");

        let assert_macros_version = get_assert_macros_version()?.to_string();

        let scarb_toml_path = dir.child("Scarb.toml");
        scarb_toml_path.write_str(&formatdoc!(
            r#"
                [package]
                name = "test_package"
                version = "0.1.0"

                [dependencies]
                starknet = "2.4.0"
                snforge_std = {{ path = "{}" }}
                assert_macros = "{}"
                "#,
            snforge_std_path,
            assert_macros_version
        ))?;

        Ok(Self {
            dir,
            contracts,
            environment_variables: HashMap::new(),
        })
    }

    pub fn set_env(&mut self, key: &str, value: &str) {
        self.environment_variables.insert(key.into(), value.into());
    }

    #[must_use]
    pub fn env(&self) -> &HashMap<String, String> {
        &self.environment_variables
    }

    pub fn path(&self) -> Result<Utf8PathBuf> {
        Utf8PathBuf::from_path_buf(self.dir.path().to_path_buf())
            .map_err(|_| anyhow!("Failed to convert TestCase path to Utf8PathBuf"))
    }

    #[must_use]
    pub fn linked_libraries(&self) -> Vec<LinkedLibrary> {
        let snforge_std_path = PathBuf::from_str("../../snforge_std")
            .unwrap()
            .canonicalize()
            .unwrap();
        vec![
            LinkedLibrary {
                name: Self::PACKAGE_NAME.to_string(),
                path: self.dir.path().join("src"),
            },
            LinkedLibrary {
                name: "snforge_std".to_string(),
                path: snforge_std_path.join("src"),
            },
        ]
    }

    pub fn contracts(&self) -> Result<HashMap<String, (StarknetContractArtifacts, Utf8PathBuf)>> {
        self.contracts
            .clone()
            .into_iter()
            .map(|contract| {
                let name = contract.name.clone();
                let (sierra, casm) = contract.generate_sierra_and_casm()?;

                Ok((
                    name,
                    (
                        StarknetContractArtifacts { sierra, casm },
                        Utf8PathBuf::default(),
                    ),
                ))
            })
            .collect()
    }

    #[must_use]
    pub fn find_test_result(results: &[TestTargetSummary]) -> &TestTargetSummary {
        results
            .iter()
            .find(|tc| !tc.test_case_summaries.is_empty())
            .unwrap()
    }
}

#[macro_export]
macro_rules! test_case {
    ( $test_code:expr ) => ({
        use $crate::runner::TestCase;
        TestCase::from($test_code, vec![]).unwrap()
    });
    ( $test_code:expr, $( $contract:expr ),*) => ({
        use $crate::runner::TestCase;

        let contracts = vec![$($contract,)*];
        TestCase::from($test_code, contracts).unwrap()
    });
}

pub fn assert_passed(result: &[TestTargetSummary]) {
    let result = &TestCase::find_test_result(result).test_case_summaries;

    assert!(!result.is_empty(), "No test results found");
    assert!(
        result.iter().all(AnyTestCaseSummary::is_passed),
        "Some tests didn't pass"
    );
}

pub fn assert_failed(result: &[TestTargetSummary]) {
    let result = &TestCase::find_test_result(result).test_case_summaries;

    assert!(!result.is_empty(), "No test results found");
    assert!(
        result.iter().all(AnyTestCaseSummary::is_failed),
        "Some tests didn't fail"
    );
}

pub fn assert_case_output_contains(
    result: &[TestTargetSummary],
    test_case_name: &str,
    asserted_msg: &str,
) {
    let test_name_suffix = format!("::{test_case_name}");

    let result = TestCase::find_test_result(result);

    assert!(result.test_case_summaries.iter().any(|any_case| {
        if any_case.is_passed() || any_case.is_failed() {
            return any_case.msg().unwrap().contains(asserted_msg)
                && any_case
                    .name()
                    .unwrap()
                    .ends_with(test_name_suffix.as_str());
        }
        false
    }));
}

pub fn assert_gas(result: &[TestTargetSummary], test_case_name: &str, asserted_gas: GasVector) {
    let test_name_suffix = format!("::{test_case_name}");

    let result = TestCase::find_test_result(result);

    assert!(result.test_case_summaries.iter().any(|any_case| {
        match any_case {
            AnyTestCaseSummary::Fuzzing(_) => {
                panic!("Cannot use assert_gas! for fuzzing tests")
            }
            AnyTestCaseSummary::Single(case) => match case {
                TestCaseSummary::Passed { gas_info: gas, .. } => {
                    println!("Gas:          {gas:?}");
                    println!("Asserted gas: {gas:?}");
                    assert_gas_with_margin(*gas, asserted_gas)
                        && any_case
                            .name()
                            .unwrap()
                            .ends_with(test_name_suffix.as_str())
                }
                _ => false,
            },
        }
    }));
}

// This logic is used to assert exact gas values in CI for the minimal supported Scarb version
// and to assert gas values with a margin in scheduled tests, as values can vary for different Scarb versions
// FOR LOCAL DEVELOPMENT ALWAYS USE EXACT CALCULATIONS
fn assert_gas_with_margin(gas: GasVector, asserted_gas: GasVector) -> bool {
    if cfg!(feature = "assert_non_exact_gas") {
        let diff = gas_vector_abs_diff(&gas, &asserted_gas);
        println!("Gas diff: {diff:?}");
        diff.l1_gas.0 <= 10 && diff.l1_data_gas.0 <= 10 && diff.l2_gas.0 <= 200_000
    } else {
        let equal = gas == asserted_gas;
        let l1_gas_equal = gas.l1_gas == asserted_gas.l1_gas;
        println!("{} == {}: {l1_gas_equal}", gas.l1_gas, asserted_gas.l1_gas);
        let l1_data_gas_equal = gas.l1_data_gas == asserted_gas.l1_data_gas;
        println!("l1_data_gas == asserted_l1_data_gas: {l1_data_gas_equal}");
        let l2_gas_equal = gas.l2_gas == asserted_gas.l2_gas;
        println!("l2_gas == asserted_l2_gas: {l2_gas_equal}");
        println!("gas == asserted_gas: {equal}");
        gas == asserted_gas
    }
}

fn gas_vector_abs_diff(a: &GasVector, b: &GasVector) -> GasVector {
    GasVector {
        l1_gas: GasAmount(a.l1_gas.0.abs_diff(b.l1_gas.0)),
        l1_data_gas: GasAmount(a.l1_data_gas.0.abs_diff(b.l1_data_gas.0)),
        l2_gas: GasAmount(a.l2_gas.0.abs_diff(b.l2_gas.0)),
    }
}

pub fn assert_syscall(
    result: &[TestTargetSummary],
    test_case_name: &str,
    syscall: DeprecatedSyscallSelector,
    expected_count: usize,
) {
    let test_name_suffix = format!("::{test_case_name}");

    let result = TestCase::find_test_result(result);

    assert!(result.test_case_summaries.iter().any(|any_case| {
        match any_case {
            AnyTestCaseSummary::Fuzzing(_) => {
                panic!("Cannot use assert_syscall! for fuzzing tests")
            }
            AnyTestCaseSummary::Single(case) => match case {
                TestCaseSummary::Passed { used_resources, .. } => {
                    used_resources
                        .syscall_usage
                        .get(&syscall)
                        .unwrap_or(&SyscallUsage::new(0, 0))
                        .call_count
                        == expected_count
                        && any_case
                            .name()
                            .unwrap()
                            .ends_with(test_name_suffix.as_str())
                }
                _ => false,
            },
        }
    }));
}

pub fn assert_builtin(
    result: &[TestTargetSummary],
    test_case_name: &str,
    builtin: BuiltinName,
    expected_count: usize,
) {
    // TODO(#2806)
    let expected_count = if builtin == BuiltinName::range_check {
        expected_count - 1
    } else {
        expected_count
    };

    let test_name_suffix = format!("::{test_case_name}");
    let result = TestCase::find_test_result(result);

    assert!(result.test_case_summaries.iter().any(|any_case| {
        match any_case {
            AnyTestCaseSummary::Fuzzing(_) => {
                panic!("Cannot use assert_builtin for fuzzing tests")
            }
            AnyTestCaseSummary::Single(case) => match case {
                TestCaseSummary::Passed { used_resources, .. } => {
                    used_resources
                        .execution_resources
                        .builtin_instance_counter
                        .get(&builtin)
                        .unwrap_or(&0)
                        == &expected_count
                        && any_case
                            .name()
                            .unwrap()
                            .ends_with(test_name_suffix.as_str())
                }
                _ => false,
            },
        }
    }));
}
