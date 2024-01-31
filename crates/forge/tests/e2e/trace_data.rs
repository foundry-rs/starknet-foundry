use std::fs;

use forge_runner::trace_data::{ProfilerCallTrace, TRACE_DIR};

use crate::e2e::common::runner::{setup_package, test_runner};

#[test]
fn simple_package_save_trace() {
    let temp = setup_package("simple_package");
    let snapbox = test_runner();
    snapbox
        .current_dir(&temp)
        .arg("--save-trace-data")
        .assert()
        .code(1);

    assert!(temp
        .join(TRACE_DIR)
        .join("simple_package::tests::test_fib.json")
        .exists());
    assert!(!temp
        .join(TRACE_DIR)
        .join("tests::test_simple::test_failing.json")
        .exists());
    assert!(!temp
        .join(TRACE_DIR)
        .join("simple_package::tests::ignored_test.json")
        .exists());
    assert!(temp
        .join(TRACE_DIR)
        .join("tests::ext_function_test::test_simple.json")
        .exists());

    let trace_data = fs::read_to_string(
        temp.join(TRACE_DIR)
            .join("tests::ext_function_test::test_simple.json"),
    )
    .unwrap();

    let call_trace: ProfilerCallTrace =
        serde_json::from_str(&trace_data).expect("Failed to parse call_trace");

    assert!(call_trace.nested_calls.is_empty());

    // Check if it doesn't crash in case some data already exists
    let snapbox = test_runner();
    snapbox
        .current_dir(&temp)
        .arg("--save-trace-data")
        .assert()
        .code(1);
}
