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

#[test]
fn trace_has_contract_names() {
    let temp = setup_package("trace");
    let snapbox = test_runner();

    snapbox
        .arg("--save-trace-data")
        .current_dir(&temp)
        .assert()
        .success();

    let trace_data = fs::read_to_string(
        temp.join(TRACE_DIR)
            .join("tests::test_trace::test_trace_print.json"),
    )
    .unwrap();

    let call_trace: ProfilerCallTrace =
        serde_json::from_str(&trace_data).expect("Failed to parse call_trace");

    assert_eq!(
        call_trace.entry_point.contract_name,
        Some(String::from("SNFORGE_TEST_CODE"))
    );
    assert_contract_names(&call_trace.nested_calls[0]);
}

fn assert_contract_names(trace: &ProfilerCallTrace) {
    // every call in this package uses the same contract
    assert_eq!(
        trace.entry_point.contract_name,
        Some(String::from("SimpleContract"))
    );

    for sub_trace in &trace.nested_calls {
        assert_contract_names(sub_trace);
    }
}
