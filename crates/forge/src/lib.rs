use anyhow::{anyhow, Result};
use camino::{Utf8Path, Utf8PathBuf};
use itertools::Itertools;
use rand::{thread_rng, RngCore};
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;

use scarb_artifacts::StarknetContractArtifacts;
use test_collector::{LinkedLibrary, RawForkConfig, RawForkParams, TestCase};
use tokio_util::sync::CancellationToken;

use crate::collecting::{
    collect_test_compilation_targets, compile_tests, CompiledTestCrate, CompiledTestCrateRaw,
    CompiledTestCrateRunnable, ValidatedForkConfig,
};
use crate::scarb::config::{ForgeConfig, ForkTarget};
use crate::test_filter::TestsFilter;

pub mod collecting;
pub mod pretty_printing;
pub mod scarb;

mod test_filter;

const FUZZER_RUNS_DEFAULT: u32 = 256;
pub const CACHE_DIR: &str = ".snfoundry_cache";

/// Configuration of the test runner
#[derive(Debug, PartialEq)]
pub struct RunnerConfig {
    pub workspace_root: Utf8PathBuf,
    pub exit_first: bool,
    pub tests_filter: TestsFilter,
    fork_targets: Vec<ForkTarget>,
    pub fuzzer_runs: u32,
    pub fuzzer_seed: u64,
}

pub struct CancellationTokens {
    exit_first: CancellationToken,
    error: CancellationToken,
}

impl CancellationTokens {
    #[must_use]
    pub fn new() -> Self {
        let exit_first = CancellationToken::new();
        let error = CancellationToken::new();
        Self { exit_first, error }
    }
}

impl Default for CancellationTokens {
    fn default() -> Self {
        Self::new()
    }
}

impl RunnerConfig {
    /// Creates a new `RunnerConfig` from given arguments
    ///
    /// # Arguments
    ///
    /// * `test_name_filter` - Used to filter test cases by names
    /// * `exact_match` - Should test names match the `test_name_filter` exactly
    /// * `exit_first` - Should runner exit after first failed test
    #[must_use]
    #[allow(clippy::too_many_arguments, clippy::fn_params_excessive_bools)]
    pub fn new(
        workspace_root: Utf8PathBuf,
        test_name_filter: Option<String>,
        exact_match: bool,
        exit_first: bool,
        only_ignored: bool,
        include_ignored: bool,
        fuzzer_runs: Option<u32>,
        fuzzer_seed: Option<u64>,
        forge_config_from_scarb: &ForgeConfig,
    ) -> Self {
        Self {
            workspace_root,
            exit_first: forge_config_from_scarb.exit_first || exit_first,
            fork_targets: forge_config_from_scarb.fork.clone(),
            fuzzer_runs: fuzzer_runs
                .or(forge_config_from_scarb.fuzzer_runs)
                .unwrap_or(FUZZER_RUNS_DEFAULT),
            fuzzer_seed: fuzzer_seed
                .or(forge_config_from_scarb.fuzzer_seed)
                .unwrap_or_else(|| thread_rng().next_u64()),
            tests_filter: TestsFilter::from_flags(
                test_name_filter,
                exact_match,
                only_ignored,
                include_ignored,
            ),
        }
    }
}

pub struct RunnerParams {
    corelib_path: Utf8PathBuf,
    pub contracts: HashMap<String, StarknetContractArtifacts>,
    pub predeployed_contracts: Utf8PathBuf,
    pub environment_variables: HashMap<String, String>,
    linked_libraries: Vec<LinkedLibrary>,
}

impl RunnerParams {
    #[must_use]
    pub fn new(
        corelib_path: Utf8PathBuf,
        contracts: HashMap<String, StarknetContractArtifacts>,
        predeployed_contracts: Utf8PathBuf,
        environment_variables: HashMap<String, String>,
        linked_libraries: Vec<LinkedLibrary>,
    ) -> Self {
        Self {
            corelib_path,
            contracts,
            predeployed_contracts,
            environment_variables,
            linked_libraries,
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum CrateLocation {
    /// Main crate in a package
    Lib,
    /// Crate in the `tests/` directory
    Tests,
}

fn parse_fork_params(raw_fork_params: &RawForkParams) -> Result<ValidatedForkConfig> {
    Ok(ValidatedForkConfig {
        url: raw_fork_params.url.parse()?,
        block_id: raw_fork_params.block_id,
    })
}

fn replace_id_with_params(
    raw_fork_config: RawForkConfig,
    runner_config: &RunnerConfig,
) -> Result<RawForkParams> {
    match raw_fork_config {
        RawForkConfig::Params(raw_fork_params) => Ok(raw_fork_params),
        RawForkConfig::Id(name) => {
            let fork_target_from_runner_config = runner_config
                .fork_targets
                .iter()
                .find(|fork| fork.name == name)
                .ok_or_else(|| {
                    anyhow!("Fork configuration named = {name} not found in the Scarb.toml")
                })?;

            Ok(fork_target_from_runner_config.params.clone())
        }
    }
}

fn to_runnable(
    compiled_test_crate: CompiledTestCrateRaw,
    runner_config: &RunnerConfig,
) -> Result<CompiledTestCrateRunnable> {
    let mut test_cases = vec![];

    for case in compiled_test_crate.test_cases {
        let fork_config = if let Some(fc) = case.fork_config {
            let raw_fork_params = replace_id_with_params(fc, runner_config)?;
            let validated_fork_config = parse_fork_params(&raw_fork_params)?;
            Some(validated_fork_config)
        } else {
            None
        };

        test_cases.push(TestCase {
            name: case.name,
            available_gas: case.available_gas,
            ignored: case.ignored,
            expected_result: case.expected_result,
            fork_config,
            fuzzer_config: case.fuzzer_config,
        });
    }

    Ok(CompiledTestCrate {
        sierra_program: compiled_test_crate.sierra_program,
        test_cases,
        tests_location: compiled_test_crate.tests_location,
    })
}

/// Run the tests in the package at the given path
///
/// # Arguments
///
/// * `package_path` - Absolute path to the top-level of the Cairo package
/// * `lib_path` - Absolute path to the main file in the package (usually `src/lib.cairo`)
/// * `linked_libraries` - Dependencies needed to run the package at `package_path`
/// * `runner_config` - A configuration of the test runner
/// * `corelib_path` - Absolute path to the Cairo corelib
/// * `contracts` - Map with names of contract used in tests and corresponding sierra and casm artifacts
/// * `predeployed_contracts` - Absolute path to predeployed contracts used by starknet state e.g. account contracts
///

#[allow(clippy::implicit_hasher)]
pub async fn run(
    package_path: &Utf8Path,
    package_name: &str,
    package_source_dir_path: &Utf8Path,
    runner_config: Arc<RunnerConfig>,
    runner_params: Arc<RunnerParams>,
    cancellation_tokens: Arc<CancellationTokens>,
) -> Result<Vec<TestCrateSummary>> {
    let compilation_targets =
        collect_test_compilation_targets(package_path, package_name, package_source_dir_path)?;
    let test_crates = compile_tests(&compilation_targets, &runner_params)?;
    let test_crates = test_crates
        .into_iter()
        .map(|tc| runner_config.tests_filter.filter_tests(tc))
        .collect_vec();
    let test_crates = test_crates
        .into_iter()
        .map(|ctc| to_runnable(ctc, &runner_config))
        .collect::<Result<Vec<_>>>()?;

    pretty_printing::print_collected_tests_count(
        test_crates.iter().map(|tests| tests.test_cases.len()).sum(),
        package_name,
    );

    let mut summaries = vec![];

    for compiled_test_crate in test_crates {
        let compiled_test_crate = Arc::new(compiled_test_crate);
        let runner_config = runner_config.clone();
        let runner_params = runner_params.clone();
        let cancellation_tokens = cancellation_tokens.clone();

        pretty_printing::print_running_tests(
            compiled_test_crate.tests_location,
            compiled_test_crate.test_cases.len(),
        );

        let summary = forge_runner::run_tests_from_crate(
            compiled_test_crate,
            runner_config,
            runner_params,
            cancellation_tokens,
        )
        .await?;

        summaries.push(summary);
    }

    pretty_printing::print_test_summary(&summaries);

    if summaries
        .iter()
        .any(|summary| summary.contained_fuzzed_tests)
    {
        pretty_printing::print_test_seed(runner_config.fuzzer_seed);
    }

    Ok(summaries)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_filter::{IgnoredFilter, NameFilter};
    use cairo_lang_sierra::program::Program;
    use starknet::core::types::BlockId;
    use starknet::core::types::BlockTag::Latest;
    use test_collector::ExpectedTestResult;

    #[test]
    fn fuzzer_default_seed() {
        let workspace_root: Utf8PathBuf = Default::default();
        let config = RunnerConfig::new(
            workspace_root.clone(),
            None,
            false,
            false,
            false,
            false,
            None,
            None,
            &Default::default(),
        );
        let config2 = RunnerConfig::new(
            workspace_root,
            None,
            false,
            false,
            false,
            false,
            None,
            None,
            &Default::default(),
        );

        assert_ne!(config.fuzzer_seed, 0);
        assert_ne!(config2.fuzzer_seed, 0);
        assert_ne!(config.fuzzer_seed, config2.fuzzer_seed);
    }

    #[test]
    fn runner_config_default_arguments() {
        let workspace_root: Utf8PathBuf = Default::default();
        let config = RunnerConfig::new(
            workspace_root.clone(),
            None,
            false,
            false,
            false,
            false,
            None,
            None,
            &Default::default(),
        );
        assert_eq!(
            config,
            RunnerConfig {
                workspace_root,
                exit_first: false,
                tests_filter: TestsFilter {
                    name_filter: NameFilter::All,
                    ignored_filter: IgnoredFilter::NotIgnored
                },
                fork_targets: vec![],
                fuzzer_runs: FUZZER_RUNS_DEFAULT,
                fuzzer_seed: config.fuzzer_seed,
            }
        );
    }

    #[test]
    fn runner_config_just_scarb_arguments() {
        let config_from_scarb = ForgeConfig {
            exit_first: true,
            fork: vec![],
            fuzzer_runs: Some(1234),
            fuzzer_seed: Some(500),
        };
        let workspace_root: Utf8PathBuf = Default::default();

        let config = RunnerConfig::new(
            workspace_root.clone(),
            Some("test".to_string()),
            false,
            false,
            true,
            false,
            None,
            None,
            &config_from_scarb,
        );
        assert_eq!(
            config,
            RunnerConfig {
                workspace_root,
                exit_first: true,
                fork_targets: vec![],
                tests_filter: TestsFilter {
                    name_filter: NameFilter::Match("test".to_string()),
                    ignored_filter: IgnoredFilter::Ignored
                },
                fuzzer_runs: 1234,
                fuzzer_seed: 500,
            }
        );
    }

    #[test]
    fn runner_config_argument_precedence() {
        let workspace_root: Utf8PathBuf = Default::default();

        let config_from_scarb = ForgeConfig {
            exit_first: false,
            fork: vec![],
            fuzzer_runs: Some(1234),
            fuzzer_seed: Some(1000),
        };
        let config = RunnerConfig::new(
            workspace_root.clone(),
            Some("abc".to_string()),
            true,
            true,
            false,
            true,
            Some(100),
            Some(32),
            &config_from_scarb,
        );
        assert_eq!(
            config,
            RunnerConfig {
                workspace_root,
                exit_first: true,
                tests_filter: TestsFilter {
                    name_filter: NameFilter::ExactMatch("abc".to_string()),
                    ignored_filter: IgnoredFilter::All
                },
                fork_targets: vec![],
                fuzzer_runs: 100,
                fuzzer_seed: 32,
            }
        );
    }

    #[test]
    #[should_panic]
    fn only_ignored_and_include_ignored_both_true() {
        let _ = RunnerConfig::new(
            Default::default(),
            None,
            false,
            false,
            true,
            true,
            None,
            None,
            &Default::default(),
        );
    }

    #[test]
    #[should_panic]
    fn exact_match_true_without_test_filter_name() {
        let _ = RunnerConfig::new(
            Default::default(),
            None,
            true,
            false,
            true,
            true,
            None,
            None,
            &Default::default(),
        );
    }

    #[test]
    fn to_runnable_unparsable_url() {
        let mocked_tests = CompiledTestCrate {
            sierra_program: Program {
                type_declarations: vec![],
                libfunc_declarations: vec![],
                statements: vec![],
                funcs: vec![],
            },
            test_cases: vec![TestCase {
                name: "crate1::do_thing".to_string(),
                available_gas: None,
                ignored: false,
                expected_result: ExpectedTestResult::Success,
                fork_config: Some(RawForkConfig::Params(RawForkParams {
                    url: "unparsable_url".to_string(),
                    block_id: BlockId::Tag(Latest),
                })),
                fuzzer_config: None,
            }],
            tests_location: CrateLocation::Lib,
        };
        let config = RunnerConfig::new(
            Default::default(),
            None,
            false,
            false,
            false,
            false,
            None,
            None,
            &Default::default(),
        );

        assert!(to_runnable(mocked_tests, &config).is_err());
    }

    #[test]
    fn to_runnable_non_existent_id() {
        let mocked_tests = CompiledTestCrate {
            sierra_program: Program {
                type_declarations: vec![],
                libfunc_declarations: vec![],
                statements: vec![],
                funcs: vec![],
            },
            test_cases: vec![TestCase {
                name: "crate1::do_thing".to_string(),
                available_gas: None,
                ignored: false,
                expected_result: ExpectedTestResult::Success,
                fork_config: Some(RawForkConfig::Id("non_existent".to_string())),
                fuzzer_config: None,
            }],
            tests_location: CrateLocation::Lib,
        };
        let config = RunnerConfig::new(
            Default::default(),
            None,
            false,
            false,
            false,
            false,
            None,
            None,
            &ForgeConfig {
                exit_first: false,
                fuzzer_runs: None,
                fuzzer_seed: None,
                fork: vec![ForkTarget {
                    name: "definitely_non_existing".to_string(),
                    params: RawForkParams {
                        url: "https://not_taken.com".to_string(),
                        block_id: BlockId::Number(120),
                    },
                }],
            },
        );

        assert!(to_runnable(mocked_tests, &config).is_err());
    }
}
