use assert_fs::TempDir;
use std::fs;

use forge_runner::trace_data::ProfilerDeprecatedSyscallSelector::{
    EmitEvent, GetBlockHash, GetExecutionInfo, Keccak, SendMessageToL1, StorageRead, StorageWrite,
};
use forge_runner::trace_data::{ProfilerCallTrace, ProfilerExecutionResources, TRACE_DIR};

use crate::e2e::common::runner::{setup_package, test_runner};

#[test]
fn trace_resources() {
    let temp = setup_package("trace_resources");
    let snapbox = test_runner();
    snapbox
        .current_dir(&temp)
        .arg("--save-trace-data")
        .assert()
        .success();

    for test_name in [
        "test_call",
        "test_deploy",
        // "test_failed_call",
        // "test_failed_lib_call",
        "test_l1_handler",
        "test_lib_call",
    ] {
        let call_trace = deserialize_call_trace(test_name, &temp);
        ensure_resources_are_correct(&call_trace);
    }
}

fn deserialize_call_trace(test_name: &str, temp_dir: &TempDir) -> ProfilerCallTrace {
    let trace_data = fs::read_to_string(
        temp_dir
            .join(TRACE_DIR)
            .join(format!("tests::{test_name}::{test_name}.json")),
    )
    .unwrap();
    serde_json::from_str(&trace_data).expect("Failed to parse call trace")
}

fn ensure_resources_are_correct(call_trace: &ProfilerCallTrace) -> &ProfilerExecutionResources {
    let mut child_resources = vec![];
    for call in &call_trace.nested_calls {
        child_resources.push(ensure_resources_are_correct(call));
    }

    let mut sum_child_resources = ProfilerExecutionResources::default();
    for resource in child_resources {
        sum_child_resources += resource;
    }

    let current_resources = &call_trace.used_resources.execution_resources;
    assert!(current_resources.gt_eq_than(&sum_child_resources));
    let resource_diff = current_resources - &sum_child_resources;
    assert_correct_diff_for_easily_countable_syscalls(&resource_diff);

    &call_trace.used_resources.execution_resources
}

fn assert_correct_diff_for_easily_countable_syscalls(resource_diff: &ProfilerExecutionResources) {
    for syscall in [
        EmitEvent,
        GetBlockHash,
        GetExecutionInfo,
        StorageWrite,
        StorageRead,
        SendMessageToL1,
        Keccak,
    ] {
        assert_eq!(
            *resource_diff
                .syscall_counter
                .get(&syscall)
                .unwrap_or_else(|| panic!("Expected resource diff to contain {syscall:?}")),
            1,
            "Incorrect diff for {syscall:?}"
        );
    }

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
