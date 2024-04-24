use crate::runner::TestCase;
use camino::Utf8PathBuf;
use cheatnet::runtime_extensions::forge_runtime_extension::contracts_data::ContractsData;
use forge::block_number_map::BlockNumberMap;
use forge::run;
use forge::scarb::{get_test_artifacts_path, load_test_artifacts};
use forge::test_filter::TestsFilter;
use forge_runner::forge_config::{
    ExecutionDataToSave, ForgeConfig, OutputConfig, SierraTestCodePathConfig, TestRunnerConfig,
};
use forge_runner::test_crate_summary::TestCrateSummary;
use forge_runner::{CACHE_DIR, SIERRA_TEST_CODE_DIR};
use shared::command::CommandExt;
use std::num::NonZeroU32;
use std::process::Command;
use std::process::Stdio;
use std::sync::Arc;
use tempfile::tempdir;
use tokio::runtime::Runtime;

#[must_use]
pub fn run_test_case(test: &TestCase) -> Vec<TestCrateSummary> {
    Command::new("scarb")
        .current_dir(test.path().unwrap())
        .arg("snforge-test-collector")
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output_checked()
        .unwrap();

    let rt = Runtime::new().expect("Could not instantiate Runtime");
    let test_artifacts_path = get_test_artifacts_path(
        &test.path().unwrap().join("target/dev/snforge"),
        "test_package",
    );
    let compiled_test_crates = load_test_artifacts(&test_artifacts_path).unwrap();

    rt.block_on(run(
        compiled_test_crates,
        "test_package",
        &TestsFilter::from_flags(None, false, false, false, false, Default::default()),
        Arc::new(ForgeConfig {
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
            }),
            sierra_test_code_path_config: SierraTestCodePathConfig {
                package_name: "test_package".to_string(),
                sierra_test_code_dir: Utf8PathBuf::from_path_buf(tempdir().unwrap().into_path())
                    .unwrap()
                    .join(SIERRA_TEST_CODE_DIR),
            },
        }),
        &[],
        &mut BlockNumberMap::default(),
    ))
    .expect("Runner fail")
}
