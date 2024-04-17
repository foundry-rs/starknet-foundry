use crate::runner::TestCase;
use camino::Utf8PathBuf;
use cheatnet::runtime_extensions::forge_runtime_extension::contracts_data::ContractsData;
use forge::block_number_map::BlockNumberMap;
use forge::run;
use forge::scarb::{get_test_artifacts_path, load_test_artifacts};
use forge::test_filter::TestsFilter;
use forge_runner::context_data::{ContextData, RuntimeData};
use forge_runner::forge_config::{
    ExecutionDataToSave, ForgeConfig, OutputConfig, RunnerConfig, RuntimeConfig,
};
use forge_runner::test_crate_summary::TestCrateSummary;
use shared::command::CommandExt;
use std::num::NonZeroU32;
use std::path::PathBuf;
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
            runner_config: Arc::new(RunnerConfig {
                exit_first: false,
                fuzzer_runs: NonZeroU32::new(256).unwrap(),
                fuzzer_seed: 12345,
            }),
            runtime_config: Arc::new(RuntimeConfig {
                max_n_steps: None,
                is_vm_trace_needed: false,
            }),
            output_config: OutputConfig {
                detailed_resources: false,
                execution_data_to_save: ExecutionDataToSave::None,
            },
        }),
        Arc::new(ContextData {
            runtime_data: RuntimeData {
                contracts_data: ContractsData::try_from(test.contracts().unwrap()).unwrap(),
                environment_variables: test.env().clone(),
            },
            workspace_root: Utf8PathBuf::from_path_buf(PathBuf::from(tempdir().unwrap().path()))
                .unwrap(),
            test_artifacts_path,
        }),
        &[],
        &mut BlockNumberMap::default(),
    ))
    .expect("Runner fail")
}
