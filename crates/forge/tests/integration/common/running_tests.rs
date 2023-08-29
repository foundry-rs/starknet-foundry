use crate::integration::common::corelib::{corelib_path, predeployed_contracts};
use crate::integration::common::runner::TestCase;
use camino::Utf8PathBuf;
use forge::TestFileSummary;
use forge::{run, RunnerConfig};

pub fn run_test_case(test: &TestCase) -> Vec<TestFileSummary> {
    run(
        &test.path().unwrap(),
        &String::from("src"),
        &test.path().unwrap().join("src/lib.cairo"),
        &Some(test.linked_libraries()),
        &Default::default(),
        &corelib_path(),
        &test.contracts(&corelib_path()).unwrap(),
        &Utf8PathBuf::from_path_buf(predeployed_contracts().to_path_buf()).unwrap(),
    )
    .unwrap()
}

pub fn run_fork_test_case(test: &TestCase) -> Vec<TestFileSummary> {
    run(
        &test.path().unwrap(),
        &String::from("src"),
        &test.path().unwrap().join("src/lib.cairo"),
        &Some(test.linked_libraries()),
        &RunnerConfig::new(
            None,
            false,
            false,
            Some("http://188.34.188.184:9545/rpc/v0.4".to_string()),
            &Default::default(),
        ),
        &corelib_path(),
        &test.contracts(&corelib_path()).unwrap(),
        &Utf8PathBuf::from_path_buf(predeployed_contracts().to_path_buf()).unwrap(),
    )
    .unwrap()
}
