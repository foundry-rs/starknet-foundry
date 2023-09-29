use std::sync::Arc;

use crate::integration::common::corelib::{corelib_path, predeployed_contracts};
use crate::integration::common::runner::TestCase;
use camino::Utf8PathBuf;
use forge::run;
use forge::TestFileSummary;
use rand::{thread_rng, RngCore};
use tokio_util::sync::CancellationToken;

#[tokio::main]
pub async fn run_test_case(test: &TestCase) -> Vec<TestFileSummary> {
    let token = CancellationToken::new();
    let cancellation_token = Arc::new(token.clone());
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
        cancellation_token,
    )
    .await
    .unwrap()
}
