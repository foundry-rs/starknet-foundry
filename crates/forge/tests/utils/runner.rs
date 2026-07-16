use crate::utils::{
    get_assert_macros_version, get_std_name, get_std_path, tempdir_with_tool_versions,
};
use anyhow::{Context, Result, anyhow};
use assert_fs::{
    TempDir,
    fixture::{FileTouch, FileWriteStr, PathChild},
};
use blockifier::execution::syscalls::vm_syscall_utils::{SyscallSelector, SyscallUsage};
use cairo_vm::types::builtin_name::BuiltinName;
use camino::Utf8PathBuf;
use forge_runner::{
    test_case_summary::{AnyTestCaseSummary, Single, TestCaseSummary},
    test_target_summary::TestTargetSummary,
};
use foundry_ui::UI;
use indoc::formatdoc;
use scarb_api::metadata::metadata_for_dir;
use scarb_api::{
    CompilationOpts, ContractData, ContractsData, StarknetContractArtifacts,
    get_contracts_artifacts_and_source_sierra_paths, target_dir_for_workspace,
};
use shared::{command::CommandExt, utils::contract_name_from_module_path};
use starknet_api::execution_resources::{GasAmount, GasVector};
use std::{
    cell::Cell,
    collections::HashMap,
    fs,
    panic::Location,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    str::FromStr,
};

// Allowed absolute gas differences when `non_exact_gas_assertions` is enabled.
const MARGIN_L1_GAS: u64 = 10;
const MARGIN_L1_DATA_GAS: u64 = 10;
const MARGIN_L2_GAS: u64 = 200_000;
const GAS_EXPECTATIONS_ENV: &str = "SNFORGE_GAS_EXPECTATIONS";
const GAS_EXPECTATIONS_RECORD_MODE: &str = "record";
const GAS_EXPECTATION_RECORD_PREFIX: &str = "SNFORGE_GAS_EXPECTATION";

/// Represents a dependency of a Cairo project
#[derive(Debug, Clone)]
pub struct LinkedLibrary {
    pub name: String,
    pub path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct Contract {
    module_path: String,
    code: String,
}

impl Contract {
    #[must_use]
    pub fn new(module_path: impl Into<String>, code: impl Into<String>) -> Self {
        Self {
            module_path: module_path.into(),
            code: code.into(),
        }
    }

    pub fn from_code_path(module_path: impl Into<String>, path: impl AsRef<Path>) -> Result<Self> {
        let code = fs::read_to_string(path)?;
        Ok(Self {
            module_path: module_path.into(),
            code,
        })
    }

    fn generate_contract_artifacts(self, ui: &UI) -> Result<StarknetContractArtifacts> {
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

        let scarb_metadata = metadata_for_dir(dir.path())?;
        let package = scarb_metadata
            .packages
            .iter()
            .find(|package| package.name == "contract")
            .unwrap();
        let artifacts_dir = target_dir_for_workspace(&scarb_metadata).join("dev");

        let mut artifacts = get_contracts_artifacts_and_source_sierra_paths(
            &artifacts_dir,
            package,
            ui,
            CompilationOpts {
                use_test_target_contracts: false,
                #[cfg(feature = "cairo-native")]
                run_native: true,
            },
        )
        .unwrap();

        let artifacts = artifacts
            .remove(&self.module_path)
            .map(|contract| contract.artifacts)
            .ok_or(anyhow!(
                "there is no contract with module path {}",
                self.module_path
            ))?;

        Ok(artifacts)
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

        let snforge_std_name = get_std_name();
        let snforge_std_path = get_std_path().unwrap();
        let assert_macros_version = get_assert_macros_version()?.to_string();

        let scarb_toml_path = dir.child("Scarb.toml");
        scarb_toml_path.write_str(&formatdoc!(
            r#"
            [package]
            name = "test_package"
            version = "0.1.0"

            [dependencies]
            starknet = "2.4.0"
            {snforge_std_name} = {{ path = "{snforge_std_path}" }}
            assert_macros = "{assert_macros_version}"
            "#
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

    pub fn contracts(&self, ui: &UI) -> Result<ContractsData> {
        self.contracts
            .clone()
            .into_iter()
            .map(|contract| {
                let module_path = contract.module_path.clone();
                let name = contract_name_from_module_path(&module_path).to_string();
                let artifacts = contract.generate_contract_artifacts(ui)?;

                Ok((
                    module_path,
                    ContractData {
                        name,
                        artifacts,
                        sierra_path: Utf8PathBuf::default(),
                    },
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

#[expect(clippy::crate_in_macro_def)]
#[macro_export]
macro_rules! test_case {
    ( $test_code:expr ) => ({
        use crate::utils::runner::TestCase;

        TestCase::from($test_code, vec![]).unwrap()
    });
    ( $test_code:expr, $( $contract:expr ),*) => ({
        use crate::utils::runner::TestCase;

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

#[track_caller]
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

#[track_caller]
pub fn assert_gas(result: &[TestTargetSummary], test_case_name: &str, asserted_gas: GasVector) {
    let result = TestCase::find_test_result(result);
    let any_case = find_case_by_name(result, test_case_name, "Gas");

    match any_case {
        AnyTestCaseSummary::Fuzzing(_) => {
            panic!("Cannot use assert_gas! for fuzzing tests")
        }
        AnyTestCaseSummary::Single(case) => {
            if let TestCaseSummary::Passed {
                name,
                gas_info: gas,
                ..
            } = case
            {
                let actual_gas = gas.gas_used;

                emit_gas_expectation_record(
                    Location::caller(),
                    &[
                        ("kind", "gas".to_string()),
                        ("test", test_case_name.to_string()),
                        ("l1_gas", actual_gas.l1_gas.0.to_string()),
                        ("l1_data_gas", actual_gas.l1_data_gas.0.to_string()),
                        ("l2_gas", actual_gas.l2_gas.0.to_string()),
                    ],
                );

                if !assert_gas_with_margin(actual_gas, asserted_gas) {
                    let diff = gas_vector_abs_diff(&actual_gas, &asserted_gas);
                    panic!(
                        "Gas assertion failed for test case `{name}`.\nexpected: {}\nactual:   {}\ndiff:     {}{}",
                        format_gas_vector(&asserted_gas),
                        format_gas_vector(&actual_gas),
                        format_gas_vector(&diff),
                        gas_assertion_margin_message(),
                    );
                }
            } else {
                // The case was located by matching on its name, so `name()` is always `Some`.
                let name = any_case.name().expect("matched test case must have a name");
                panic!(
                    "Gas assertion failed for test case `{name}`: expected passed test case with gas information, but test case was {}",
                    test_case_status(case)
                );
            }
        }
    }
}

// This logic is used to assert exact gas values in CI for the minimal supported Scarb version
// and to assert gas values with a margin in scheduled tests, as values can vary for different Scarb versions
// FOR LOCAL DEVELOPMENT ALWAYS USE EXACT CALCULATIONS
fn assert_gas_with_margin(gas: GasVector, asserted_gas: GasVector) -> bool {
    if cfg!(feature = "non_exact_gas_assertions") {
        let diff = gas_vector_abs_diff(&gas, &asserted_gas);
        diff.l1_gas.0 <= MARGIN_L1_GAS
            && diff.l1_data_gas.0 <= MARGIN_L1_DATA_GAS
            && diff.l2_gas.0 <= MARGIN_L2_GAS
    } else {
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

fn format_gas_vector(gas: &GasVector) -> String {
    format!(
        "l1_gas: {}, l1_data_gas: {}, l2_gas: {}",
        gas.l1_gas.0, gas.l1_data_gas.0, gas.l2_gas.0
    )
}

fn gas_assertion_margin_message() -> String {
    if cfg!(feature = "non_exact_gas_assertions") {
        format!(
            "\nallowed diff: l1_gas <= {MARGIN_L1_GAS}, l1_data_gas <= {MARGIN_L1_DATA_GAS}, l2_gas <= {MARGIN_L2_GAS}"
        )
    } else {
        String::new()
    }
}

fn gas_expectation_recording_enabled() -> bool {
    std::env::var(GAS_EXPECTATIONS_ENV).is_ok_and(|value| value == GAS_EXPECTATIONS_RECORD_MODE)
}

thread_local! {
    /// When set, `emit_gas_expectation_record` is a no-op on the current thread.
    /// Used by diagnostics that deliberately trigger assertion failures (see
    /// [`without_gas_expectation_recording`]) so they don't emit misleading records
    /// that the update script would then try to apply.
    static RECORDING_SUPPRESSED: Cell<bool> = const { Cell::new(false) };
}

/// Runs `f` with gas-expectation recording suppressed on the current thread.
///
/// `libtest` runs each test on its own thread, so a thread-local flag is enough to scope
/// the suppression to a single assertion without affecting tests running in parallel.
/// The flag is reset via a drop guard, so it is cleared even if `f` panics (which is the
/// expected case for the diagnostics tests that use this).
pub fn without_gas_expectation_recording<T>(f: impl FnOnce() -> T) -> T {
    struct ResetGuard;
    impl Drop for ResetGuard {
        fn drop(&mut self) {
            RECORDING_SUPPRESSED.with(|suppressed| suppressed.set(false));
        }
    }

    RECORDING_SUPPRESSED.with(|suppressed| suppressed.set(true));
    let _guard = ResetGuard;
    f()
}

fn emit_gas_expectation_record(location: &Location<'_>, fields: &[(&str, String)]) {
    if !gas_expectation_recording_enabled() || RECORDING_SUPPRESSED.with(Cell::get) {
        return;
    }

    let mut record = format!(
        "{GAS_EXPECTATION_RECORD_PREFIX}|file={}|line={}",
        location.file(),
        location.line()
    );

    for (key, value) in fields {
        record.push('|');
        record.push_str(key);
        record.push('=');
        record.push_str(value);
    }

    eprintln!("{record}");
}

fn test_case_status(case: &TestCaseSummary<Single>) -> &'static str {
    match case {
        TestCaseSummary::Passed { .. } => "passed",
        TestCaseSummary::Failed { .. } => "failed",
        TestCaseSummary::Ignored { .. } => "ignored",
        TestCaseSummary::Interrupted { .. } => "interrupted",
        TestCaseSummary::ExcludedFromPartition { .. } => "excluded from partition",
    }
}

/// Finds the (single) test case whose fully-qualified name ends with `::{test_case_name}`.
///
/// Panics with a helpful, `assertion`-prefixed message listing the available test cases when no
/// matching case is found.
#[track_caller]
fn find_case_by_name<'a>(
    result: &'a TestTargetSummary,
    test_case_name: &str,
    assertion: &str,
) -> &'a AnyTestCaseSummary {
    let test_name_suffix = format!("::{test_case_name}");

    result
        .test_case_summaries
        .iter()
        .find(|any_case| {
            any_case
                .name()
                .is_some_and(|name| name.ends_with(test_name_suffix.as_str()))
        })
        .unwrap_or_else(|| {
            let available_test_cases = result
                .test_case_summaries
                .iter()
                .filter_map(AnyTestCaseSummary::name)
                .collect::<Vec<_>>()
                .join(", ");

            panic!(
                "{assertion} assertion failed: test case `{test_case_name}` was not found. Available test cases: {available_test_cases}"
            )
        })
}

#[track_caller]
pub fn assert_syscall(
    result: &[TestTargetSummary],
    test_case_name: &str,
    syscall: SyscallSelector,
    expected_count: usize,
) {
    let result = TestCase::find_test_result(result);
    let any_case = find_case_by_name(result, test_case_name, "Syscall");

    match any_case {
        AnyTestCaseSummary::Fuzzing(_) => {
            panic!("Cannot use assert_syscall! for fuzzing tests")
        }
        AnyTestCaseSummary::Single(case) => match case {
            TestCaseSummary::Passed { used_resources, .. } => {
                let actual_count = used_resources
                    .syscall_usage
                    .get(&syscall)
                    .unwrap_or(&SyscallUsage::new(0, 0))
                    .call_count;

                emit_gas_expectation_record(
                    Location::caller(),
                    &[
                        ("kind", "syscall".to_string()),
                        ("test", test_case_name.to_string()),
                        ("syscall", format!("{syscall:?}")),
                        ("count", actual_count.to_string()),
                    ],
                );

                // In record mode we only want to capture the actual value, without failing, so
                // that every assertion in a test emits its record in a single run.
                if !gas_expectation_recording_enabled() {
                    assert!(
                        actual_count == expected_count,
                        "Syscall assertion failed for test case `{test_case_name}`: expected {expected_count} `{syscall:?}` syscall(s), but found {actual_count}"
                    );
                }
            }
            _ => {
                let name = any_case.name().expect("matched test case must have a name");
                panic!(
                    "Syscall assertion failed for test case `{name}`: expected passed test case with resource information, but test case was {}",
                    test_case_status(case)
                );
            }
        },
    }
}

#[track_caller]
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

    let result = TestCase::find_test_result(result);
    let any_case = find_case_by_name(result, test_case_name, "Builtin");

    match any_case {
        AnyTestCaseSummary::Fuzzing(_) => {
            panic!("Cannot use assert_builtin for fuzzing tests")
        }
        AnyTestCaseSummary::Single(case) => match case {
            TestCaseSummary::Passed { used_resources, .. } => {
                let actual_count = *used_resources
                    .execution_summary
                    .charged_resources
                    .extended_vm_resources
                    .vm_resources
                    .builtin_instance_counter
                    .get(&builtin)
                    .unwrap_or(&0);
                // The value written in test sources accounts for the range check builtin used by
                // the assertion itself (see the `expected_count` adjustment above).
                let source_count = if builtin == BuiltinName::range_check {
                    actual_count + 1
                } else {
                    actual_count
                };

                emit_gas_expectation_record(
                    Location::caller(),
                    &[
                        ("kind", "builtin".to_string()),
                        ("test", test_case_name.to_string()),
                        ("builtin", format!("{builtin:?}")),
                        ("count", source_count.to_string()),
                    ],
                );

                // In record mode we only want to capture the actual value, without failing, so
                // that every assertion in a test emits its record in a single run.
                if !gas_expectation_recording_enabled() {
                    // Report counts in the source-value domain (see `source_count`) so the message
                    // matches what is written in the test.
                    let expected_source = if builtin == BuiltinName::range_check {
                        expected_count + 1
                    } else {
                        expected_count
                    };
                    assert!(
                        actual_count == expected_count,
                        "Builtin assertion failed for test case `{test_case_name}`: expected {expected_source} `{builtin:?}` builtin(s), but found {source_count}"
                    );
                }
            }
            _ => {
                let name = any_case.name().expect("matched test case must have a name");
                panic!(
                    "Builtin assertion failed for test case `{name}`: expected passed test case with resource information, but test case was {}",
                    test_case_status(case)
                );
            }
        },
    }
}
