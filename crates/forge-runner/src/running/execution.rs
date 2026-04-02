use blockifier::execution::call_info::{CallExecution, CallInfo, ExtendedExecutionResources};
use blockifier::execution::contract_class::TrackedResource;
use blockifier::execution::entry_point_execution::{
    extract_extended_vm_resources, finalize_runner, get_call_result, total_vm_resources,
};
use blockifier::execution::errors::PostExecutionError;
use blockifier::execution::syscalls::hint_processor::SyscallHintProcessor;
use cairo_vm::vm::runners::cairo_runner::CairoRunner;

// Based on the code from blockifer
#[tracing::instrument(skip_all, level = "debug")]
pub fn finalize_execution(
    // region: Modified blockifier code
    runner: &mut CairoRunner,
    syscall_handler: &mut SyscallHintProcessor<'_>,
    // endregion
    n_total_args: usize,
    program_extra_data_length: usize,
    tracked_resource: TrackedResource,
) -> Result<CallInfo, PostExecutionError> {
    finalize_runner(runner, n_total_args, program_extra_data_length)?;
    syscall_handler
        .read_only_segments
        .mark_as_accessed(runner)?;

    let call_result = get_call_result(runner, syscall_handler, &tracked_resource)?;

    // Take into account the resources of the current call, without inner calls.
    // Has to happen after marking holes in segments as accessed.
    let extended_resources_without_inner_calls =
        extract_extended_vm_resources(runner, syscall_handler)?;

    let tracked_vm_resources_without_inner_calls = match tracked_resource {
        TrackedResource::CairoSteps => &extended_resources_without_inner_calls,
        TrackedResource::SierraGas => &ExtendedExecutionResources::default(),
    };

    syscall_handler.finalize();

    let vm_resources = total_vm_resources(
        tracked_vm_resources_without_inner_calls,
        &syscall_handler.base.inner_calls,
    );

    let syscall_handler_base = &syscall_handler.base;
    // region: Modified blockifier code - added clones due to different function signature
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
        storage_access_tracker: syscall_handler_base.storage_access_tracker.clone(),
        builtin_counters: extended_resources_without_inner_calls.prover_cairo_primitives(),
        // TODO: Investigate if we can reduce our logic given that syscall usage is now present in `CallInfo`
        syscalls_usage: syscall_handler_base.syscalls_usage.clone(),
    })
    // endregion
}
