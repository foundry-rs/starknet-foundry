use assert_fs::TempDir;
use std::collections::HashMap;
use std::fs;

use forge_runner::trace_data::ProfilerDeprecatedSyscallSelector::{
    CallContract, Deploy, EmitEvent, GetBlockHash, GetExecutionInfo, Keccak, LibraryCall,
    SendMessageToL1, StorageRead, StorageWrite,
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
        // TODO(#1657):
        //  "test_failed_call",
        //  "test_failed_lib_call",
        "test_l1_handler",
        "test_lib_call",
    ] {
        let call_trace = deserialize_call_trace(test_name, &temp);
        ensure_resources_are_correct(&call_trace);
    }

    // tests for Deploy, CallContract and LibraryCall syscalls as they cannot be tested as easily as the rest
    ensure_test_call_not_easily_countable_syscalls(&temp);
    ensure_test_deploy_not_easily_countable_syscalls(&temp);
    ensure_test_l1_handler_not_easily_countable_syscalls(&temp);
    ensure_test_libcall_not_easily_countable_syscalls(&temp);
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

    let current_resources = &call_trace.used_execution_resources;
    assert!(current_resources.gt_eq_than(&sum_child_resources));
    let resource_diff = current_resources - &sum_child_resources;
    assert_correct_diff_for_easily_countable_syscalls(&resource_diff);

    current_resources
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

// When sth fails in the functions below and you didn't change anything in the cairo code it is a BUG.
// If you changed the corresponding cairo code count the expected occurrences of syscalls manually first, then assert them.
// TL;DR: DON't mindlessly change numbers to fix the tests if they ever fail.
fn ensure_test_call_not_easily_countable_syscalls(temp_dir: &TempDir) {
    let test_call_trace = deserialize_call_trace("test_call", temp_dir);
    assert_not_easily_countable_syscalls(&test_call_trace, 11, 4, 1); // FIXME in #1631 (should be 14, 8, 1)

    let regular_call = &test_call_trace.nested_calls[1];
    assert_not_easily_countable_syscalls(regular_call, 2, 1, 0);

    let from_proxy = &regular_call.nested_calls[0];
    assert_not_easily_countable_syscalls(from_proxy, 1, 0, 0);

    let with_libcall = &test_call_trace.nested_calls[2];
    assert_not_easily_countable_syscalls(with_libcall, 2, 0, 1);

    let from_proxy = &with_libcall.nested_calls[0];
    assert_not_easily_countable_syscalls(from_proxy, 1, 0, 0);

    let call_two = &test_call_trace.nested_calls[3];
    assert_not_easily_countable_syscalls(call_two, 3, 2, 0);

    let from_proxy = &call_two.nested_calls[0];
    assert_not_easily_countable_syscalls(from_proxy, 1, 0, 0);

    let from_proxy_dummy = &call_two.nested_calls[1];
    assert_not_easily_countable_syscalls(from_proxy_dummy, 1, 0, 0);

    let from_proxy = &test_call_trace.nested_calls[4];
    assert_not_easily_countable_syscalls(from_proxy, 1, 0, 0);
}

fn ensure_test_deploy_not_easily_countable_syscalls(temp_dir: &TempDir) {
    let test_call_trace = deserialize_call_trace("test_deploy", temp_dir);
    assert_not_easily_countable_syscalls(&test_call_trace, 11, 4, 0); // FIXME in #1631 (should be 14, 4, 0)

    for deploy_proxy in test_call_trace.nested_calls {
        assert_not_easily_countable_syscalls(&deploy_proxy, 2, 1, 0);

        let from_proxy = &deploy_proxy.nested_calls[0];
        assert_not_easily_countable_syscalls(from_proxy, 1, 0, 0);
    }
}

fn ensure_test_l1_handler_not_easily_countable_syscalls(temp_dir: &TempDir) {
    let test_call_trace = deserialize_call_trace("test_l1_handler", temp_dir);
    assert_not_easily_countable_syscalls(&test_call_trace, 6, 3, 0); // FIXME in #1631 (should be 8, 3, 0)

    let handle_l1 = &test_call_trace.nested_calls[1];
    assert_not_easily_countable_syscalls(handle_l1, 3, 2, 0);

    let regular_call = &handle_l1.nested_calls[0];
    assert_not_easily_countable_syscalls(regular_call, 2, 1, 0);

    let from_proxy = &regular_call.nested_calls[0];
    assert_not_easily_countable_syscalls(from_proxy, 1, 0, 0);
}

fn ensure_test_libcall_not_easily_countable_syscalls(temp_dir: &TempDir) {
    let test_call_trace = deserialize_call_trace("test_lib_call", temp_dir);
    assert_not_easily_countable_syscalls(&test_call_trace, 9, 3, 1); // FIXME in #1631 (should be 11, 3, 5)

    let regular_call = &test_call_trace.nested_calls[0];
    assert_not_easily_countable_syscalls(regular_call, 2, 1, 0);

    let from_proxy = &regular_call.nested_calls[0];
    assert_not_easily_countable_syscalls(from_proxy, 1, 0, 0);

    let with_libcall = &test_call_trace.nested_calls[1];
    assert_not_easily_countable_syscalls(with_libcall, 2, 0, 1);

    let call_two = &test_call_trace.nested_calls[2];
    assert_not_easily_countable_syscalls(call_two, 3, 2, 0);

    let from_proxy = &call_two.nested_calls[0];
    assert_not_easily_countable_syscalls(from_proxy, 1, 0, 0);

    let from_proxy_dummy = &call_two.nested_calls[1];
    assert_not_easily_countable_syscalls(from_proxy_dummy, 1, 0, 0);

    let from_proxy = &test_call_trace.nested_calls[3];
    assert_not_easily_countable_syscalls(from_proxy, 1, 0, 0);
}

fn assert_not_easily_countable_syscalls(
    call: &ProfilerCallTrace,
    deploy_count: usize,
    call_contract_count: usize,
    library_call_count: usize,
) {
    let syscall_counter = &call.used_execution_resources.syscall_counter;

    let expected_counts: HashMap<_, _> = [
        (Deploy, deploy_count),
        (CallContract, call_contract_count),
        (LibraryCall, library_call_count),
    ]
    .into_iter()
    .filter(|(_key, val)| *val > 0)
    .collect();

    for key in [Deploy, CallContract, LibraryCall] {
        assert_eq!(
            syscall_counter.get(&key).copied(),
            expected_counts.get(&key).copied(),
            "Incorrect count for {key:?}"
        );
    }
}
