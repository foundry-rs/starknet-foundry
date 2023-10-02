use std::sync::Arc;

use crate::integration::common::corelib::{corelib_path, predeployed_contracts};
use crate::integration::common::runner::TestCase;
use camino::Utf8PathBuf;

use forge::run;
use forge::TestFileSummary;
use rand::{thread_rng, RngCore};
use tokio_util::sync::CancellationToken;

use forge::{run, RunnerConfig, RunnerParams, TestCrateSummary};
use std::default::Default;
use std::path::PathBuf;
use tempfile::tempdir;

#[tokio::main]
pub fn run_test_case(test: &TestCase) -> Vec<TestCrateSummary> {
    let token = CancellationToken::new();
    let cancellation_token = Arc::new(token.clone());
    run(
        &Utf8PathBuf::from_path_buf(PathBuf::from(tempdir().unwrap().path())).unwrap(),
        &test.path().unwrap(),
        &String::from("src"),
        &test.path().unwrap().join("src"),
        &test.linked_libraries(),
        &RunnerConfig::new(
            None,
            false,
            false,
            Some(256),
            Some(12345),
            &Default::default(),
        ),
        can & RunnerParams::new(
            corelib_path(),
            test.contracts(&corelib_path()).unwrap(),
            Utf8PathBuf::from_path_buf(predeployed_contracts().to_path_buf()).unwrap(),
            test.env().clone(),
        ),
        cancellation_token,
    )
    .await
    .unwrap()
}
