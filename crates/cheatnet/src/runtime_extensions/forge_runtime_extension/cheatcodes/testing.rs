use std::{cell::RefCell, rc::Rc};

use blockifier::{
    blockifier_versioned_constants::VersionedConstants,
    execution::syscalls::vm_syscall_utils::SyscallUsageMap,
};
use cairo_vm::vm::runners::cairo_runner::ExecutionResources;

use crate::{
    runtime_extensions::common::sum_syscall_usage,
    trace_data::{CallTrace, CallTraceNode},
};

pub fn calculate_steps_from_calls(
    top_call: &Rc<RefCell<CallTrace>>,
    top_call_syscalls: &SyscallUsageMap,
) -> usize {
    let used_resources =
        &top_call
            .borrow()
            .nested_calls
            .iter()
            .fold(ExecutionResources::default(), |acc, node| match node {
                CallTraceNode::EntryPointCall(call_trace) => {
                    &acc + &call_trace.borrow().used_execution_resources
                }
                CallTraceNode::DeployWithoutConstructor => acc,
            });

    let inner_calls_syscalls = &top_call.borrow().nested_calls.iter().fold(
        SyscallUsageMap::new(),
        |acc, node| match node {
            CallTraceNode::EntryPointCall(call_trace) => sum_syscall_usage(
                acc,
                &sum_syscall_usage(
                    call_trace.borrow().used_syscalls_sierra_gas.clone(),
                    &call_trace.borrow().used_syscalls_vm_resources,
                ),
            ),
            CallTraceNode::DeployWithoutConstructor => acc,
        },
    );

    let total_syscalls = sum_syscall_usage(inner_calls_syscalls.clone(), top_call_syscalls);

    let total_syscalls_exeucution_resources = &VersionedConstants::latest_constants()
        .get_additional_os_syscall_resources(&total_syscalls);

    let resources_from_calls = used_resources + total_syscalls_exeucution_resources;

    resources_from_calls.n_steps
}
