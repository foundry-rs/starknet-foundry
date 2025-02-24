use crate::runner::TestCase;
use camino::Utf8PathBuf;
use cheatnet::runtime_extensions::forge_runtime_extension::contracts_data::ContractsData;
use forge::{
    block_number_map::BlockNumberMap,
    run_tests::package::{RunForPackageArgs, run_for_package},
    scarb::load_test_artifacts,
    test_filter::TestsFilter,
};
use forge_runner::CACHE_DIR;
use forge_runner::forge_config::{
    ExecutionDataToSave, ForgeConfig, OutputConfig, TestRunnerConfig,
};
use forge_runner::test_target_summary::TestTargetSummary;
use scarb_api::{ScarbCommand, metadata::MetadataCommandExt};
use std::num::NonZeroU32;
use std::sync::Arc;
use tempfile::tempdir;
use tokio::runtime::Runtime;

#[must_use]
pub fn run_test_case(test: &TestCase) -> Vec<TestTargetSummary> {
    ScarbCommand::new_with_stdio()
        .current_dir(test.path().unwrap())
        .arg("build")
        .arg("--test")
        .run()
        .unwrap();

    let metadata = ScarbCommand::metadata()
        .current_dir(test.path().unwrap())
        .run()
        .unwrap();

    let package = metadata
        .packages
        .iter()
        .find(|p| p.name == "test_package")
        .unwrap();

    let rt = Runtime::new().expect("Could not instantiate Runtime");
    let raw_test_targets =
        load_test_artifacts(&test.path().unwrap().join("target/dev"), package).unwrap();

    rt.block_on(run_for_package(
        RunForPackageArgs {
            test_targets: raw_test_targets,
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
                    execution_data_to_save: ExecutionDataToSave::default(),
                }),
            }),
            fork_targets: vec![],
        },
        &mut BlockNumberMap::default(),
    ))
    .expect("Runner fail")
}
