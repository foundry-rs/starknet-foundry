use crate::runner::TestCase;
use camino::Utf8PathBuf;
use cheatnet::runtime_extensions::forge_runtime_extension::contracts_data::ContractsData;
use forge::block_number_map::BlockNumberMap;
use forge::run;
use forge::scarb::{get_test_artifacts_path, load_test_artifacts};
use forge::test_filter::TestsFilter;
use forge_runner::test_crate_summary::TestCrateSummary;
use forge_runner::{RunnerConfig, RunnerParams};
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
        Arc::new(RunnerConfig::new(
            Utf8PathBuf::from_path_buf(PathBuf::from(tempdir().unwrap().path())).unwrap(),
            false,
            NonZeroU32::new(256).unwrap(),
            12345,
            false,
            false,
            false,
            None,
        )),
        Arc::new(RunnerParams::new(
            ContractsData::try_from(test.contracts().unwrap()).unwrap(),
            test.env().clone(),
        )),
        &[],
        &mut BlockNumberMap::default(),
    ))
    .expect("Runner fail")
}
