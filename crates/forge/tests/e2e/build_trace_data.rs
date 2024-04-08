use super::common::runner::{setup_package, test_runner};
use crate::e2e::common::get_trace_from_trace_node;
use forge_runner::build_trace_data::{TEST_CODE_CONTRACT_NAME, TEST_CODE_FUNCTION_NAME, TRACE_DIR};
use std::fs;
use trace_data::{CallTrace as ProfilerCallTrace, CallTraceNode as ProfilerCallTraceNode};

#[test]
fn simple_package_save_trace() {
    let temp = setup_package("simple_package");
    test_runner(&temp).arg("--save-trace-data").assert().code(1);

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
    test_runner(&temp).arg("--save-trace-data").assert().code(1);
}

#[test]
fn trace_has_contract_and_function_names() {
    let temp = setup_package("trace");
    test_runner(&temp)
        .arg("--save-trace-data")
        .assert()
        .success();

    let trace_data = fs::read_to_string(
        temp.join(TRACE_DIR)
            .join("tests::test_trace::test_trace.json"),
    )
    .unwrap();

    let call_trace: ProfilerCallTrace =
        serde_json::from_str(&trace_data).expect("Failed to parse call_trace");

    assert_eq!(
        call_trace.entry_point.contract_name,
        Some(String::from(TEST_CODE_CONTRACT_NAME))
    );
    assert_eq!(
        call_trace.entry_point.function_name,
        Some(String::from(TEST_CODE_FUNCTION_NAME))
    );
    assert_contract_and_function_names(get_trace_from_trace_node(&call_trace.nested_calls[3]));
}

fn assert_contract_and_function_names(trace: &ProfilerCallTrace) {
    // every call in this package uses the same contract and function
    assert_eq!(
        trace.entry_point.contract_name,
        Some(String::from("SimpleContract"))
    );
    assert_eq!(
        trace.entry_point.function_name,
        Some(String::from("execute_calls"))
    );

    for sub_trace_node in &trace.nested_calls {
        assert_contract_and_function_names(get_trace_from_trace_node(sub_trace_node));
    }
}

#[test]
fn trace_has_vm_trace() {
    let temp = setup_package("trace");
    let snapbox = test_runner(&temp);

    snapbox
        .arg("--save-trace-data")
        .current_dir(&temp)
        .assert()
        .success();

    let trace_data = fs::read_to_string(
        temp.join(TRACE_DIR)
            .join("tests::test_trace::test_trace.json"),
    )
    .unwrap();

    let call_trace: ProfilerCallTrace =
        serde_json::from_str(&trace_data).expect("Failed to parse call_trace");

    assert_vm_trace_exists(&call_trace);
}

fn assert_vm_trace_exists(trace: &ProfilerCallTrace) {
    assert!(
        trace.vm_trace.is_some() || trace.entry_point.function_name == Some(String::from("fail"))
    );

    for sub_trace_node in &trace.nested_calls {
        if let ProfilerCallTraceNode::EntryPointCall(sub_trace) = sub_trace_node {
            assert_vm_trace_exists(sub_trace);
        }
    }
}

#[test]
fn trace_has_deploy_with_no_constructor_phantom_nodes() {
    let temp = setup_package("trace");
    let snapbox = test_runner(&temp);

    snapbox
        .arg("--save-trace-data")
        .current_dir(&temp)
        .assert()
        .success();

    let trace_data = fs::read_to_string(
        temp.join(TRACE_DIR)
            .join("tests::test_trace::test_trace.json"),
    )
    .unwrap();

    let call_trace: ProfilerCallTrace =
        serde_json::from_str(&trace_data).expect("Failed to parse call_trace");

    // 3 first calls are deploys with empty constructors
    matches!(
        call_trace.nested_calls[0],
        trace_data::CallTraceNode::DeployWithoutConstructor
    );
    matches!(
        call_trace.nested_calls[1],
        trace_data::CallTraceNode::DeployWithoutConstructor
    );
    matches!(
        call_trace.nested_calls[2],
        trace_data::CallTraceNode::DeployWithoutConstructor
    );
}
