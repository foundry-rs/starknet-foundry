use crate::running::copied_code::{
    extract_vm_resources, finalize_runner, get_call_result, total_vm_resources,
};
use blockifier::execution::call_info::{CallExecution, CallInfo};
use blockifier::execution::contract_class::TrackedResource;
use blockifier::execution::errors::PostExecutionError;
use blockifier::execution::syscalls::hint_processor::SyscallHintProcessor;
use blockifier::transaction::objects::ExecutionResourcesTraits;
use cairo_vm::vm::runners::cairo_runner::CairoRunner;

// Based on the code from blockifer
pub fn finalize_execution(
    // region: Modified blockifier code
    runner: &mut CairoRunner,
    syscall_handler: &mut SyscallHintProcessor<'_>,
    // endregion
    n_total_args: usize,
    program_extra_data_length: usize,
    tracked_resource: TrackedResource,
) -> Result<CallInfo, PostExecutionError> {
    // region: Modified blockifier code
    finalize_runner(runner, n_total_args, program_extra_data_length)?;
    syscall_handler
        .read_only_segments
        .mark_as_accessed(runner)?;
    // endregion

    let call_result = get_call_result(runner, syscall_handler, &tracked_resource)?;

    let vm_resources_without_inner_calls =
        extract_vm_resources(runner, syscall_handler, tracked_resource)?;

    syscall_handler.finalize();

    let vm_resources = total_vm_resources(
        &vm_resources_without_inner_calls,
        &syscall_handler.base.inner_calls,
    );

    let syscall_handler_base = &syscall_handler.base;
    Ok(CallInfo {
        call: syscall_handler_base.call.clone().into(),
        execution: CallExecution {
            retdata: call_result.retdata,
            events: syscall_handler_base.events.clone(),
            l2_to_l1_messages: syscall_handler_base.l2_to_l1_messages.clone(),
            cairo_native: false,
            failed: call_result.failed,
            gas_consumed: call_result.gas_consumed,
        },
        inner_calls: syscall_handler_base.inner_calls.clone(),
        tracked_resource,
        resources: vm_resources,
        storage_read_values: syscall_handler_base.read_values.clone(),
        accessed_storage_keys: syscall_handler_base.accessed_keys.clone(),
        read_class_hash_values: syscall_handler_base.read_class_hash_values.clone(),
        accessed_contract_addresses: syscall_handler_base.accessed_contract_addresses.clone(),
        builtin_counters: vm_resources_without_inner_calls.prover_builtins(),
    })
}
