use crate::runner::TestCase;
use camino::Utf8PathBuf;
use cheatnet::runtime_extensions::forge_runtime_extension::contracts_data::ContractsData;
use forge::{
    block_number_map::BlockNumberMap,
    run_tests::package::{run_from_package, RunFromCrateArgs},
    scarb::load_test_artifacts,
    test_filter::TestsFilter,
};
use forge_runner::forge_config::{
    ExecutionDataToSave, ForgeConfig, OutputConfig, TestRunnerConfig,
};
use forge_runner::test_crate_summary::TestTargetSummary;
use forge_runner::CACHE_DIR;
use forge_runner::{
    build_trace_data::test_sierra_program_path::VERSIONED_PROGRAMS_DIR,
    package_tests::raw::TestCrateRaw,
};
use shared::command::CommandExt;
use std::num::NonZeroU32;
use std::process::Command;
use std::process::Stdio;
use std::sync::Arc;
use tempfile::tempdir;
use tokio::runtime::Runtime;

#[must_use]
pub fn run_test_case(test: &TestCase) -> Vec<TestTargetSummary> {
    Command::new("scarb")
        .current_dir(test.path().unwrap())
        .arg("snforge-test-collector")
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output_checked()
        .unwrap();

    let rt = Runtime::new().expect("Could not instantiate Runtime");
    let compiled_test_crates = load_test_artifacts(
        &test.path().unwrap().join("target/dev/snforge"),
        "test_package",
    )
    .unwrap();

    rt.block_on(run_from_package(
        RunFromCrateArgs {
            test_targets: compiled_test_crates
                .into_iter()
                .map(TestCrateRaw::with_config)
                .collect(),
            package_name: "test_package".to_string(),
            tests_filter: TestsFilter::from_flags(
                None,
                false,
                false,
                false,
                false,
                Default::default(),
            ),
            forge_config: Arc::new(ForgeConfig {
                test_runner_config: Arc::new(TestRunnerConfig {
                    exit_first: false,
                    fuzzer_runs: NonZeroU32::new(256).unwrap(),
                    fuzzer_seed: 12345,
                    max_n_steps: None,
                    is_vm_trace_needed: false,
                    cache_dir: Utf8PathBuf::from_path_buf(tempdir().unwrap().into_path())
                        .unwrap()
                        .join(CACHE_DIR),
                    contracts_data: ContractsData::try_from(test.contracts().unwrap()).unwrap(),
                    environment_variables: test.env().clone(),
                }),
                output_config: Arc::new(OutputConfig {
                    detailed_resources: false,
                    execution_data_to_save: ExecutionDataToSave::None,
                    versioned_programs_dir: Utf8PathBuf::from_path_buf(
                        tempdir().unwrap().into_path(),
                    )
                    .unwrap()
                    .join(VERSIONED_PROGRAMS_DIR),
                }),
            }),
            fork_targets: vec![],
        },
        &mut BlockNumberMap::default(),
    ))
    .expect("Runner fail")
}
