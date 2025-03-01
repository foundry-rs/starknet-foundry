use super::common::runner::{setup_package, test_runner};
use crate::e2e::common::get_trace_from_trace_node;
use cairo_annotations::trace_data::{
    CallTraceNode as ProfilerCallTraceNode, CallTraceV1 as ProfilerCallTrace,
    VersionedCallTrace as VersionedProfilerCallTrace,
};
use cairo_lang_sierra::program::VersionedProgram;
use cairo_lang_starknet_classes::contract_class::ContractClass;
use forge_runner::build_trace_data::{TEST_CODE_CONTRACT_NAME, TEST_CODE_FUNCTION_NAME, TRACE_DIR};
use std::fs;

#[test]
fn simple_package_save_trace() {
    let temp = setup_package("simple_package");
    test_runner(&temp).arg("--save-trace-data").assert().code(1);

    assert!(
        temp.join(TRACE_DIR)
            .join("simple_package_tests_test_fib.json")
            .exists()
    );
    assert!(
        !temp
            .join(TRACE_DIR)
            .join("simple_package_integrationtest_test_simple_test_failing.json")
            .exists()
    );
    assert!(
        !temp
            .join(TRACE_DIR)
            .join("simple_package_tests_ignored_test.json")
            .exists()
    );
    assert!(
        temp.join(TRACE_DIR)
            .join("simple_package_integrationtest_ext_function_test_test_simple.json")
            .exists()
    );

    let trace_data = fs::read_to_string(
        temp.join(TRACE_DIR)
            .join("simple_package_integrationtest_ext_function_test_test_simple.json"),
    )
    .unwrap();

    let VersionedProfilerCallTrace::V1(call_trace) =
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
            .join("trace_info_integrationtest_test_trace_test_trace.json"),
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
fn trace_has_cairo_execution_info() {
    let temp = setup_package("trace");
    let snapbox = test_runner(&temp);

    snapbox
        .arg("--save-trace-data")
        .current_dir(&temp)
        .assert()
        .success();

    let trace_data = fs::read_to_string(
        temp.join(TRACE_DIR)
            .join("trace_info_integrationtest_test_trace_test_trace.json"),
    )
    .unwrap();

    let call_trace: ProfilerCallTrace =
        serde_json::from_str(&trace_data).expect("Failed to parse call_trace");

    assert_cairo_execution_info_exists(&call_trace);
}

fn assert_cairo_execution_info_exists(trace: &ProfilerCallTrace) {
    if let Some(cairo_execution_info) = trace.cairo_execution_info.as_ref() {
        let sierra_string = fs::read_to_string(&cairo_execution_info.source_sierra_path).unwrap();

        assert!(
            serde_json::from_str::<VersionedProgram>(&sierra_string).is_ok()
                || serde_json::from_str::<ContractClass>(&sierra_string).is_ok()
        );
    } else {
        assert_eq!(trace.entry_point.function_name, Some(String::from("fail")));
    }

    for sub_trace_node in &trace.nested_calls {
        if let ProfilerCallTraceNode::EntryPointCall(sub_trace) = sub_trace_node {
            assert_cairo_execution_info_exists(sub_trace);
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
            .join("trace_info_integrationtest_test_trace_test_trace.json"),
    )
    .unwrap();

    let call_trace: ProfilerCallTrace =
        serde_json::from_str(&trace_data).expect("Failed to parse call_trace");

    // 3 first calls are deploys with empty constructors
    matches!(
        call_trace.nested_calls[0],
        cairo_annotations::trace_data::CallTraceNode::DeployWithoutConstructor
    );
    matches!(
        call_trace.nested_calls[1],
        cairo_annotations::trace_data::CallTraceNode::DeployWithoutConstructor
    );
    matches!(
        call_trace.nested_calls[2],
        cairo_annotations::trace_data::CallTraceNode::DeployWithoutConstructor
    );
}

#[test]
fn trace_is_produced_even_if_contract_panics() {
    let temp = setup_package("backtrace_panic");
    test_runner(&temp)
        .arg("--save-trace-data")
        .assert()
        .success();

    let trace_data = fs::read_to_string(
        temp.join(TRACE_DIR)
            .join("backtrace_panic_Test_test_contract_panics.json"),
    )
    .unwrap();

    let call_trace: ProfilerCallTrace = serde_json::from_str(&trace_data).unwrap();

    assert_all_execution_info_exists(&call_trace);
}

fn assert_all_execution_info_exists(trace: &ProfilerCallTrace) {
    assert!(trace.cairo_execution_info.is_some());

    for trace_node in &trace.nested_calls {
        if let ProfilerCallTraceNode::EntryPointCall(trace) = trace_node {
            assert_all_execution_info_exists(trace);
        }
    }
}
