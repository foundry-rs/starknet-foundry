use crate::utils::runner::TestCase;
use camino::Utf8PathBuf;
use cheatnet::runtime_extensions::forge_runtime_extension::contracts_data::ContractsData;
use forge::run_tests::package::run_for_package;
use forge::run_tests::resolve_config::resolve_config;
use forge::shared_cache::FailedTestsCache;
use forge::{
    block_number_map::BlockNumberMap, scarb::load_test_artifacts, test_filter::TestsFilter,
};
use forge_runner::CACHE_DIR;
use forge_runner::debugging::TraceArgs;
use forge_runner::forge_config::{
    ExecutionDataToSave, ForgeConfig, ForgeTrackedResource, OutputConfig, TestRunnerConfig,
};
use forge_runner::running::with_config::test_target_with_tests;
use forge_runner::test_target_summary::TestTargetSummary;
use foundry_ui::UI;
use scarb_api::ScarbCommand;
use scarb_api::metadata::metadata_for_dir;
use std::num::NonZeroU32;
use std::sync::Arc;
use tempfile::tempdir;
use tokio::runtime::Runtime;

#[must_use]
pub fn run_test_case(
    test: &TestCase,
    tracked_resource: ForgeTrackedResource,
) -> Vec<TestTargetSummary> {
    ScarbCommand::new_with_stdio()
        .current_dir(test.path().unwrap())
        .arg("build")
        .arg("--test")
        .run()
        .unwrap();

    let metadata = metadata_for_dir(test.path().unwrap()).unwrap();

    let package = metadata
        .packages
        .iter()
        .find(|p| p.name == "test_package")
        .unwrap();

    let rt = Runtime::new().expect("Could not instantiate Runtime");
    let raw_test_targets =
        load_test_artifacts(&test.path().unwrap().join("target/dev"), package).unwrap();

    let fork_targets = vec![];
    let mut block_number_map = BlockNumberMap::default();
    let tests_filter = TestsFilter::from_flags(
        None,
        false,
        Vec::new(),
        false,
        false,
        false,
        FailedTestsCache::default(),
    );

    let mut test_targets_resolved = Vec::new();

    for raw in raw_test_targets.into_iter() {
        let tt = test_target_with_tests(raw).expect("failed to prepare test target");

        let tt_resolved = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(resolve_config(
                tt,
                &fork_targets,
                &mut block_number_map,
                &tests_filter,
                &tracked_resource,
            ));
        test_targets_resolved.push(tt_resolved.unwrap());
    }

    let ui = Arc::new(UI::default());
    rt.block_on(run_for_package(
        "test_package".to_string(),
        Arc::new(ForgeConfig {
            test_runner_config: Arc::new(TestRunnerConfig {
                exit_first: false,
                fuzzer_runs: NonZeroU32::new(256).unwrap(),
                fuzzer_seed: 12345,
                max_n_steps: None,
                is_vm_trace_needed: false,
                cache_dir: Utf8PathBuf::from_path_buf(tempdir().unwrap().keep())
                    .unwrap()
                    .join(CACHE_DIR),
                contracts_data: ContractsData::try_from(test.contracts(&ui).unwrap()).unwrap(),
                tracked_resource: ForgeTrackedResource::CairoSteps,
                environment_variables: test.env().clone(),
                experimental_oracles: false,
            }),
            output_config: Arc::new(OutputConfig {
                detailed_resources: false,
                execution_data_to_save: ExecutionDataToSave::default(),
                trace_args: TraceArgs::default(),
            }),
        }),
        test_targets_resolved,
        &tests_filter,
        ui,
    ))
    .expect("Runner fail")
    .summaries()
}
