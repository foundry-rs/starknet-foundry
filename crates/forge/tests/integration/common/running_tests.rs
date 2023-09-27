use crate::integration::common::corelib::{corelib_path, predeployed_contracts};
use crate::integration::common::runner::TestCase;
use ark_std::rand::{thread_rng, RngCore};
use camino::Utf8PathBuf;
use forge::TestFileSummary;
use forge::{run, RunnerParams};
use rand::{thread_rng, RngCore};

pub fn run_test_case(test: &TestCase) -> Vec<TestFileSummary> {
    run(
        &test.path().unwrap(),
        &String::from("src"),
        &test.path().unwrap().join("src/lib.cairo"),
        &Some(test.linked_libraries()),
        &Default::default(),
        &RunnerParams::new(
            corelib_path(),
            test.contracts(&corelib_path()).unwrap(),
            Utf8PathBuf::from_path_buf(predeployed_contracts().to_path_buf()).unwrap(),
            test.env().clone(),
        ),
        thread_rng().next_u64(),
    )
    .unwrap()
}
