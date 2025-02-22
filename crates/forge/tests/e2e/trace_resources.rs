use super::common::runner::{setup_package, test_runner};
use assert_fs::TempDir;
use cairo_annotations::trace_data::{
    CallTraceV1 as ProfilerCallTrace, ExecutionResources as ProfilerExecutionResources,
    VersionedCallTrace as VersionedProfilerCallTrace,
};
use forge_runner::build_trace_data::TRACE_DIR;
use std::fs;

#[test]
fn trace_resources_call() {
    assert_resources_for_test("test_call");
}

#[test]
fn trace_resources_deploy() {
    assert_resources_for_test("test_deploy");
}

#[test]
fn trace_resources_l1_handler() {
    assert_resources_for_test("test_l1_handler");
}

#[test]
fn trace_resources_lib_call() {
    assert_resources_for_test("test_lib_call");
}

#[test]
#[ignore] // TODO(#1657)
fn trace_resources_failed_call() {
    assert_resources_for_test("test_failed_call");
}

#[test]
#[ignore] // TODO(#1657)
fn trace_resources_failed_lib_call() {
    assert_resources_for_test("test_failed_lib_call");
}

fn assert_resources_for_test(test_name: &str) {
    let temp = setup_package("trace_resources");

    test_runner(&temp)
        .arg(test_name)
        .arg("--save-trace-data")
        .assert()
        .success();

    let VersionedProfilerCallTrace::V1(call_trace) = deserialize_call_trace(test_name, &temp);
    check_vm_resources_and_easily_unifiable_syscalls(&call_trace);
}

fn deserialize_call_trace(test_name: &str, temp_dir: &TempDir) -> VersionedProfilerCallTrace {
    let trace_data = fs::read_to_string(temp_dir.join(TRACE_DIR).join(format!(
        "trace_resources_tests_{test_name}_{test_name}.json"
    )))
    .unwrap();
    serde_json::from_str(&trace_data).expect("Failed to parse call trace")
}

fn check_vm_resources_and_easily_unifiable_syscalls(
    call_trace: &ProfilerCallTrace,
) -> &ProfilerExecutionResources {
    let mut child_resources = vec![];
    for call_node in &call_trace.nested_calls {
        if let cairo_annotations::trace_data::CallTraceNode::EntryPointCall(call) = call_node {
            child_resources.push(check_vm_resources_and_easily_unifiable_syscalls(call));
        }
    }

    let mut sum_child_resources = ProfilerExecutionResources::default();
    for resource in child_resources {
        sum_child_resources += resource;
    }

    let current_resources = &call_trace.cumulative_resources;
    assert!(current_resources.gt_eq_than(&sum_child_resources));
    let resource_diff = current_resources - &sum_child_resources;
    assert_correct_diff_for_builtins_and_easily_unifiable_syscalls(&resource_diff);
    assert_l2_l1_messages(call_trace);

    current_resources
}

fn assert_correct_diff_for_builtins_and_easily_unifiable_syscalls(
    resource_diff: &ProfilerExecutionResources,
) {
    for builtin in [
        "poseidon_builtin",
        "ec_op_builtin",
        "bitwise_builtin",
        "pedersen_builtin",
    ] {
        assert_eq!(
            *resource_diff
                .vm_resources
                .builtin_instance_counter
                .get(builtin)
                .unwrap_or_else(|| panic!("Expected resource diff to contain {builtin:?}")),
            1,
            "Incorrect diff for {builtin:?}"
        );
    }
}

fn assert_l2_l1_messages(call_trace: &ProfilerCallTrace) {
    assert_eq!(
        call_trace.used_l1_resources.l2_l1_message_sizes.len(),
        1,
        "Every call should have one message"
    );
    assert_eq!(
        call_trace.used_l1_resources.l2_l1_message_sizes,
        vec![2],
        "Message should have payload of length 2"
    );
}
