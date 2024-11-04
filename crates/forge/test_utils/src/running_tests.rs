use crate::runner::TestCase;
use camino::Utf8PathBuf;
use cheatnet::runtime_extensions::forge_runtime_extension::contracts_data::ContractsData;
use forge::{
    block_number_map::BlockNumberMap,
    run_tests::package::{run_for_package, RunForPackageArgs},
    scarb::load_test_artifacts,
    test_filter::TestsFilter,
};
use forge_runner::build_trace_data::test_sierra_program_path::VERSIONED_PROGRAMS_DIR;
use forge_runner::forge_config::{
    ExecutionDataToSave, ForgeConfig, OutputConfig, TestRunnerConfig,
};
use forge_runner::test_target_summary::TestTargetSummary;
use forge_runner::CACHE_DIR;
use scarb_api::{metadata::MetadataCommandExt, ScarbCommand};
use std::num::NonZeroU32;
use std::sync::Arc;
use tempfile::tempdir;
use tokio::runtime::Runtime;

#[must_use]
pub fn run_test_case(test: &TestCase) -> Vec<TestTargetSummary> {
    let scarb_result = ScarbCommand::new_with_stdio()
        .current_dir(test.path().unwrap())
        .arg("build")
        .arg("--test")
        .run();

    if let Err(_e) = scarb_result {
        return vec![TestTargetSummary {
            test_case_summaries: vec![],
        }];
    }

    let metadata = match ScarbCommand::metadata()
        .current_dir(test.path().unwrap())
        .run()
    {
        Ok(metadata) => metadata,
        Err(_) => return vec![TestTargetSummary {
            test_case_summaries: vec![],
        }],
    };

    let package = match metadata
        .packages
        .iter()
        .find(|p| p.name == "test_package")
    {
        Some(package) => package,
        None => return vec![TestTargetSummary {
            test_case_summaries: vec![],
        }],
    };

    let rt = Runtime::new().expect("Could not instantiate Runtime");
    let raw_test_targets = match load_test_artifacts(&test.path().unwrap().join("target/dev"), package) {
        Ok(targets) => targets,
        Err(_) => return vec![TestTargetSummary {
            test_case_summaries: vec![],
        }],
    };

    match rt.block_on(run_for_package(
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
    )) {
        Ok(summaries) => summaries,
        Err(_) => vec![TestTargetSummary {
            test_case_summaries: vec![],
        }],
    }
}
