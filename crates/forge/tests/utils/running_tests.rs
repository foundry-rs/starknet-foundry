use crate::utils::runner::TestCase;
use camino::Utf8PathBuf;
use cheatnet::runtime_extensions::forge_runtime_extension::contracts_data::ContractsData;
use forge::shared_cache::FailedTestsCache;
use forge::{
    block_number_map::BlockNumberMap,
    run_tests::package::{RunForPackageArgs, run_for_package},
    run_tests::test_target::ExitFirstChannel,
    test_filter::TestsFilter,
};
use forge_runner::CACHE_DIR;
use forge_runner::debugging::TraceArgs;
use forge_runner::forge_config::{
    ExecutionDataToSave, ForgeConfig, ForgeTrackedResource, OutputConfig, TestRunnerConfig,
};
use forge_runner::partition::PartitionConfig;
use forge_runner::running::target::prepare_test_target;
use forge_runner::scarb::load_test_artifacts;
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

    let ui = Arc::new(UI::default());
    rt.block_on(async {
        let target_handles = raw_test_targets
            .into_iter()
            .map(|t| tokio::task::spawn_blocking(move || prepare_test_target(t, &tracked_resource)))
            .collect();
        run_for_package(
            RunForPackageArgs {
                target_handles,
                package_name: "test_package".to_string(),
                package_root: Utf8PathBuf::default(),
                tests_filter: TestsFilter::from_flags(
                    None,
                    false,
                    Vec::new(),
                    false,
                    false,
                    false,
                    FailedTestsCache::default(),
                    PartitionConfig::default(),
                ),
                forge_config: Arc::new(ForgeConfig {
                    test_runner_config: Arc::new(TestRunnerConfig {
                        exit_first: false,
                        deterministic_output: false,
                        fuzzer_runs: NonZeroU32::new(256).unwrap(),
                        fuzzer_seed: 12345,
                        max_n_steps: None,
                        is_vm_trace_needed: false,
                        cache_dir: Utf8PathBuf::from_path_buf(tempdir().unwrap().keep())
                            .unwrap()
                            .join(CACHE_DIR),
                        contracts_data: ContractsData::try_from(test.contracts(&ui).unwrap())
                            .unwrap(),
                        tracked_resource,
                        environment_variables: test.env().clone(),
                        launch_debugger: false,
                    }),
                    output_config: Arc::new(OutputConfig {
                        detailed_resources: false,
                        execution_data_to_save: ExecutionDataToSave::default(),
                        trace_args: TraceArgs::default(),
                        gas_report: false,
                    }),
                }),
                fork_targets: vec![],
            },
            &BlockNumberMap::default(),
            ui,
            &mut ExitFirstChannel::default(),
        )
        .await
    })
    .expect("Runner fail")
    .summaries()
}
