use std::sync::Arc;

use crate::integration::common::corelib::{corelib_path, predeployed_contracts};
use crate::integration::common::runner::TestCase;
use camino::Utf8PathBuf;

use forge::{run, CancellationTokens, RunnerConfig, RunnerParams, TestCrateSummary};
use std::default::Default;
use std::path::PathBuf;
use tempfile::tempdir;
use tokio::runtime::Runtime;
use tokio::sync::mpsc::channel;

pub fn run_test_case(test: &TestCase) -> Vec<TestCrateSummary> {
    let rt = Runtime::new().expect("Could not instantiate Runtime");
    let (send, mut r) = channel(1);

    rt.block_on(run(
        &test.path().unwrap(),
        &String::from("src"),
        &test.path().unwrap().join("src"),
        &test.linked_libraries(),
        Arc::new(RunnerConfig::new(
            Utf8PathBuf::from_path_buf(PathBuf::from(tempdir().unwrap().path())).unwrap(),
            None,
            false,
            false,
            Some(256),
            Some(12345),
            &Default::default(),
        )),
        Arc::new(RunnerParams::new(
            corelib_path(),
            test.contracts(&corelib_path()).unwrap(),
            Utf8PathBuf::from_path_buf(predeployed_contracts().to_path_buf()).unwrap(),
            test.env().clone(),
        )),
        Arc::new(CancellationTokens::new()),
        send.clone(),
    ))
    .expect("Runner fail")
}
