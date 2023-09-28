use crate::integration::common::corelib::{corelib_path, predeployed_contracts};
use crate::integration::common::runner::TestCase;
use camino::Utf8PathBuf;
use forge::{run, RunnerConfig, RunnerParams, TestCrateSummary};
use std::default::Default;
use std::path::PathBuf;
use tempfile::tempdir;

pub fn run_test_case(test: &TestCase) -> Vec<TestCrateSummary> {
    run(
        &Utf8PathBuf::from_path_buf(PathBuf::from(tempdir().unwrap().path())).unwrap(),
        &test.path().unwrap(),
        &String::from("src"),
        &test.path().unwrap().join("src/lib.cairo"),
        test.linked_libraries(),
        &RunnerConfig::new(
            None,
            false,
            false,
            Some(256),
            Some(12345),
            &Default::default(),
        ),
        &RunnerParams::new(
            corelib_path(),
            test.contracts(&corelib_path()).unwrap(),
            Utf8PathBuf::from_path_buf(predeployed_contracts().to_path_buf()).unwrap(),
            test.env().clone(),
        ),
    )
    .unwrap()
}
