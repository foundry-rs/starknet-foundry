use crate::integration::common::corelib::{corelib_path, predeployed_contracts};
use crate::integration::common::runner::TestCase;
use camino::Utf8PathBuf;
use forge::run;
use forge::TestFileSummary;
use rand::{thread_rng, RngCore};

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
        thread_rng().next_u64(),
    )
    .unwrap()
}
