use assert_fs::TempDir;
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
    ensure_test_call_syscalls(&temp);
    ensure_test_deploy_syscalls(&temp);
    ensure_test_l1_handler_syscalls(&temp);
    ensure_test_libcall_syscalls(&temp);
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
fn ensure_test_deploy_syscalls(temp_dir: &TempDir) {
    let test_call_trace = deserialize_call_trace("test_call", temp_dir);
    let top_call_syscalls = test_call_trace
        .used_resources
        .execution_resources
        .syscall_counter;
    assert_eq!(*top_call_syscalls.get(&Deploy).unwrap(), 11); // FIXME in #1631 (should be 14)
    assert_eq!(*top_call_syscalls.get(&CallContract).unwrap(), 4); // FIXME in #1631 (should be 8)
    assert_eq!(*top_call_syscalls.get(&LibraryCall).unwrap(), 1);
    let regular_call = &test_call_trace.nested_calls[1];
    let regular_call_syscalls = &regular_call
        .used_resources
        .execution_resources
        .syscall_counter;
    assert_eq!(*regular_call_syscalls.get(&Deploy).unwrap(), 2);
    assert_eq!(*regular_call_syscalls.get(&CallContract).unwrap(), 1);
    assert!(regular_call_syscalls.get(&LibraryCall).is_none());
    let from_proxy = &regular_call.nested_calls[0];
    let from_proxy_syscalls = &from_proxy
        .used_resources
        .execution_resources
        .syscall_counter;
    assert_eq!(*from_proxy_syscalls.get(&Deploy).unwrap(), 1);
    assert!(from_proxy_syscalls.get(&CallContract).is_none());
    assert!(from_proxy_syscalls.get(&LibraryCall).is_none());

    let with_libcall = &test_call_trace.nested_calls[2];
    let with_libcall_syscalls = &with_libcall
        .used_resources
        .execution_resources
        .syscall_counter;
    assert_eq!(*with_libcall_syscalls.get(&Deploy).unwrap(), 2);
    assert!(from_proxy_syscalls.get(&CallContract).is_none());
    assert_eq!(*with_libcall_syscalls.get(&LibraryCall).unwrap(), 1);
    let from_proxy = &with_libcall.nested_calls[0];
    let from_proxy_syscalls = &from_proxy
        .used_resources
        .execution_resources
        .syscall_counter;
    assert_eq!(*from_proxy_syscalls.get(&Deploy).unwrap(), 1);
    assert!(from_proxy_syscalls.get(&CallContract).is_none());
    assert!(from_proxy_syscalls.get(&LibraryCall).is_none());

    let call_two = &test_call_trace.nested_calls[3];
    let call_two_syscalls = &call_two.used_resources.execution_resources.syscall_counter;
    assert_eq!(*call_two_syscalls.get(&Deploy).unwrap(), 3);
    assert_eq!(*call_two_syscalls.get(&CallContract).unwrap(), 2);
    assert!(call_two_syscalls.get(&LibraryCall).is_none());
    let from_proxy = &call_two.nested_calls[0];
    let from_proxy_syscalls = &from_proxy
        .used_resources
        .execution_resources
        .syscall_counter;
    assert_eq!(*from_proxy_syscalls.get(&Deploy).unwrap(), 1);
    assert!(from_proxy_syscalls.get(&CallContract).is_none());
    assert!(from_proxy_syscalls.get(&LibraryCall).is_none());
    let from_proxy_dummy = &call_two.nested_calls[1];
    let from_proxy_dummy_syscalls = &from_proxy_dummy
        .used_resources
        .execution_resources
        .syscall_counter;
    assert_eq!(*from_proxy_dummy_syscalls.get(&Deploy).unwrap(), 1);
    assert!(from_proxy_dummy_syscalls.get(&CallContract).is_none());
    assert!(from_proxy_dummy_syscalls.get(&LibraryCall).is_none());

    let from_proxy = &test_call_trace.nested_calls[4];
    let from_proxy_syscalls = &from_proxy
        .used_resources
        .execution_resources
        .syscall_counter;
    assert_eq!(*from_proxy_syscalls.get(&Deploy).unwrap(), 1);
    assert!(from_proxy_syscalls.get(&CallContract).is_none());
    assert!(from_proxy_syscalls.get(&LibraryCall).is_none());
}

fn ensure_test_call_syscalls(temp_dir: &TempDir) {
    let test_call_trace = deserialize_call_trace("test_deploy", temp_dir);
    let top_call_syscalls = test_call_trace
        .used_resources
        .execution_resources
        .syscall_counter;
    assert_eq!(*top_call_syscalls.get(&Deploy).unwrap(), 11); // FIXME in #1631 (should be 14)
    assert_eq!(*top_call_syscalls.get(&CallContract).unwrap(), 4);
    assert!(top_call_syscalls.get(&LibraryCall).is_none());

    for deploy_proxy in test_call_trace.nested_calls {
        let deploy_proxy_syscalls = deploy_proxy
            .used_resources
            .execution_resources
            .syscall_counter;
        assert_eq!(*deploy_proxy_syscalls.get(&Deploy).unwrap(), 2);
        assert_eq!(*deploy_proxy_syscalls.get(&CallContract).unwrap(), 1);
        assert!(deploy_proxy_syscalls.get(&LibraryCall).is_none());

        let from_proxy = &deploy_proxy.nested_calls[0];
        let from_proxy_syscalls = &from_proxy
            .used_resources
            .execution_resources
            .syscall_counter;
        assert_eq!(*from_proxy_syscalls.get(&Deploy).unwrap(), 1);
        assert!(from_proxy_syscalls.get(&CallContract).is_none());
        assert!(from_proxy_syscalls.get(&LibraryCall).is_none());
    }
}

fn ensure_test_l1_handler_syscalls(temp_dir: &TempDir) {
    let test_call_trace = deserialize_call_trace("test_l1_handler", temp_dir);
    let top_call_syscalls = test_call_trace
        .used_resources
        .execution_resources
        .syscall_counter;
    assert_eq!(*top_call_syscalls.get(&Deploy).unwrap(), 6); // FIXME in #1631 (should be 8)
    assert_eq!(*top_call_syscalls.get(&CallContract).unwrap(), 3);
    assert!(top_call_syscalls.get(&LibraryCall).is_none());

    let handle_l1 = &test_call_trace.nested_calls[1];
    let handle_l1_syscalls = &handle_l1.used_resources.execution_resources.syscall_counter;
    assert_eq!(*handle_l1_syscalls.get(&Deploy).unwrap(), 3);
    assert_eq!(*handle_l1_syscalls.get(&CallContract).unwrap(), 2);
    assert!(handle_l1_syscalls.get(&LibraryCall).is_none());

    let regular_call = &handle_l1.nested_calls[0];
    let regular_call_syscalls = &regular_call
        .used_resources
        .execution_resources
        .syscall_counter;
    assert_eq!(*regular_call_syscalls.get(&Deploy).unwrap(), 2);
    assert_eq!(*regular_call_syscalls.get(&CallContract).unwrap(), 1);
    assert!(regular_call_syscalls.get(&LibraryCall).is_none());

    let from_proxy = &regular_call.nested_calls[0];
    let from_proxy_syscalls = &from_proxy
        .used_resources
        .execution_resources
        .syscall_counter;
    assert_eq!(*from_proxy_syscalls.get(&Deploy).unwrap(), 1);
    assert!(from_proxy_syscalls.get(&CallContract).is_none());
    assert!(from_proxy_syscalls.get(&LibraryCall).is_none());
}

fn ensure_test_libcall_syscalls(temp_dir: &TempDir) {
    let test_call_trace = deserialize_call_trace("test_lib_call", temp_dir);
    let top_call_syscalls = test_call_trace
        .used_resources
        .execution_resources
        .syscall_counter;
    assert_eq!(*top_call_syscalls.get(&Deploy).unwrap(), 9); // FIXME in #1631 (should be 11)
    assert_eq!(*top_call_syscalls.get(&CallContract).unwrap(), 3);
    assert_eq!(*top_call_syscalls.get(&LibraryCall).unwrap(), 1); // FIXME in #1631 (should be 5)
    let regular_call = &test_call_trace.nested_calls[0];
    let regular_call_syscalls = &regular_call
        .used_resources
        .execution_resources
        .syscall_counter;
    assert_eq!(*regular_call_syscalls.get(&Deploy).unwrap(), 2);
    assert_eq!(*regular_call_syscalls.get(&CallContract).unwrap(), 1);
    assert!(regular_call_syscalls.get(&LibraryCall).is_none());
    let from_proxy = &regular_call.nested_calls[0];
    let from_proxy_syscalls = &from_proxy
        .used_resources
        .execution_resources
        .syscall_counter;
    assert_eq!(*from_proxy_syscalls.get(&Deploy).unwrap(), 1);
    assert!(from_proxy_syscalls.get(&CallContract).is_none());
    assert!(from_proxy_syscalls.get(&LibraryCall).is_none());

    let with_libcall = &test_call_trace.nested_calls[1];
    let with_libcall_syscalls = &with_libcall
        .used_resources
        .execution_resources
        .syscall_counter;
    assert_eq!(*with_libcall_syscalls.get(&Deploy).unwrap(), 2);
    assert!(from_proxy_syscalls.get(&CallContract).is_none());
    assert_eq!(*with_libcall_syscalls.get(&LibraryCall).unwrap(), 1);
    let from_proxy = &with_libcall.nested_calls[0];
    let from_proxy_syscalls = &from_proxy
        .used_resources
        .execution_resources
        .syscall_counter;
    assert_eq!(*from_proxy_syscalls.get(&Deploy).unwrap(), 1);
    assert!(from_proxy_syscalls.get(&CallContract).is_none());
    assert!(from_proxy_syscalls.get(&LibraryCall).is_none());

    let call_two = &test_call_trace.nested_calls[2];
    let call_two_syscalls = &call_two.used_resources.execution_resources.syscall_counter;
    assert_eq!(*call_two_syscalls.get(&Deploy).unwrap(), 3);
    assert_eq!(*call_two_syscalls.get(&CallContract).unwrap(), 2);
    assert!(call_two_syscalls.get(&LibraryCall).is_none());
    let from_proxy = &call_two.nested_calls[0];
    let from_proxy_syscalls = &from_proxy
        .used_resources
        .execution_resources
        .syscall_counter;
    assert_eq!(*from_proxy_syscalls.get(&Deploy).unwrap(), 1);
    assert!(from_proxy_syscalls.get(&CallContract).is_none());
    assert!(from_proxy_syscalls.get(&LibraryCall).is_none());
    let from_proxy_dummy = &call_two.nested_calls[1];
    let from_proxy_dummy_syscalls = &from_proxy_dummy
        .used_resources
        .execution_resources
        .syscall_counter;
    assert_eq!(*from_proxy_dummy_syscalls.get(&Deploy).unwrap(), 1);
    assert!(from_proxy_dummy_syscalls.get(&CallContract).is_none());
    assert!(from_proxy_dummy_syscalls.get(&LibraryCall).is_none());

    let from_proxy = &test_call_trace.nested_calls[3];
    let from_proxy_syscalls = &from_proxy
        .used_resources
        .execution_resources
        .syscall_counter;
    assert_eq!(*from_proxy_syscalls.get(&Deploy).unwrap(), 1);
    assert!(from_proxy_syscalls.get(&CallContract).is_none());
    assert!(from_proxy_syscalls.get(&LibraryCall).is_none());
}
