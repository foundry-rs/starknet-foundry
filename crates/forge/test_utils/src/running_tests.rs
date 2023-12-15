use crate::runner::TestCase;
use camino::Utf8PathBuf;
use forge::block_number_map::BlockNumberMap;
use forge::run;
use forge::test_filter::TestsFilter;
use forge_runner::test_crate_summary::TestCrateSummary;
use forge_runner::{RunnerConfig, RunnerParams};
use std::path::PathBuf;
use std::process::Command;
use std::process::Stdio;
use std::sync::Arc;
use tempfile::tempdir;
use tokio::runtime::Runtime;

#[must_use]
pub fn run_test_case(test: &TestCase) -> Vec<TestCrateSummary> {
    let test_build_output = Command::new("scarb")
        .current_dir(test.path().unwrap())
        .arg("snforge-test-collector")
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output()
        .unwrap();
    assert!(test_build_output.status.success());

    let rt = Runtime::new().expect("Could not instantiate Runtime");

    rt.block_on(run(
        &String::from("test_package"),
        &test.path().unwrap().join("target/dev/snforge"),
        &TestsFilter::from_flags(None, false, false, false, false, Default::default()),
        Arc::new(RunnerConfig::new(
            Utf8PathBuf::from_path_buf(PathBuf::from(tempdir().unwrap().path())).unwrap(),
            false,
            256,
            12345,
        )),
        Arc::new(RunnerParams::new(
            test.contracts().unwrap(),
            test.env().clone(),
        )),
        &[],
        &mut BlockNumberMap::default(),
    ))
    .expect("Runner fail")
}
