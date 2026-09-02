use crate::runtime_extensions::cheatable_starknet_runtime_extension::CheatableStarknetRuntimeExtension;
use crate::runtime_extensions::common::get_relocated_vm_trace;
use crate::runtime_extensions::outer_call_runtime_extension::CheatnetState;
use crate::runtime_extensions::outer_call_runtime_extension::execution::entry_point::{
    CallInfoWithExecutionData, ContractClassEntryPointExecutionResult,
    extract_trace_and_memory_and_register_errors,
};
use blockifier::execution::contract_class::{CompiledClassV1, TrackedResource};
use blockifier::execution::entry_point::ExecutableCallEntryPoint;
use blockifier::execution::entry_point_execution::{
    ExecutionRunnerMode, VmExecutionContext, finalize_execution,
    initialize_execution_context_with_runner_mode, prepare_call_arguments,
};
use blockifier::execution::syscalls::vm_syscall_utils::SyscallUsageMap;
use blockifier::{
    execution::{
        contract_class::EntryPointV1, entry_point::EntryPointExecutionContext,
        errors::EntryPointExecutionError, execution_utils::Args,
    },
    state::state_api::State,
};
use cairo_vm::vm::errors::cairo_run_errors::CairoRunError;
use cairo_vm::vm::trace::trace_entry::RelocatedTraceEntry;
use cairo_vm::{
    hint_processor::hint_processor_definition::HintProcessor,
    vm::runners::cairo_runner::{CairoArg, CairoRunner},
};
use runtime::{ExtendedRuntime, StarknetRuntime};
use starknet_types_core::felt::Felt;

// blockifier/src/execution/cairo1_execution.rs:48 (execute_entry_point_call)
#[cfg_attr(feature = "cairo-native", expect(clippy::result_large_err))]
pub(crate) fn execute_entry_point_call_cairo1(
    call: ExecutableCallEntryPoint,
    compiled_class_v1: &CompiledClassV1,
    state: &mut dyn State,
    cheatnet_state: &mut CheatnetState, // Added parameter
    context: &mut EntryPointExecutionContext,
) -> ContractClassEntryPointExecutionResult {
    let tracked_resource = *context
        .tracked_resource_stack
        .last()
        .expect("Unexpected empty tracked resource.");
    let entry_point_initial_budget = context.gas_costs().base.entry_point_initial_budget;

    let class_hash = call.class_hash;

    let VmExecutionContext {
        mut runner,
        mut syscall_handler,
        initial_syscall_ptr,
        entry_point,
        program_extra_data_length,
    } = initialize_execution_context_with_runner_mode(
        call,
        compiled_class_v1,
        state,
        context,
        ExecutionRunnerMode::Tracing,
    )?;

    let args = prepare_call_arguments(
        &syscall_handler.base.call,
        &mut runner,
        initial_syscall_ptr,
        &mut syscall_handler.read_only_segments,
        &entry_point,
        entry_point_initial_budget,
    )?;
    let n_total_args = args.len();

    // region: Modified blockifier code

    let mut cheatable_runtime = ExtendedRuntime {
        extension: CheatableStarknetRuntimeExtension { cheatnet_state },
        extended_runtime: StarknetRuntime {
            hint_handler: syscall_handler,
            panic_traceback: None,
        },
    };

    // Execute.
    cheatable_run_entry_point(
        &mut runner,
        &mut cheatable_runtime,
        &entry_point,
        &args,
        program_extra_data_length,
    )
    .map_err(|source| {
        extract_trace_and_memory_and_register_errors(
            source,
            class_hash,
            &mut runner,
            cheatable_runtime.extension.cheatnet_state,
        )
    })?;

    let trace = get_relocated_vm_trace(&mut runner);
    // Captured before `runner` is consumed by `finalize_execution` below.
    let memory = relocated_memory(&runner);

    // Syscall usage here is flat, meaning it only includes syscalls from current call
    let syscall_usage = cheatable_runtime
        .extended_runtime
        .hint_handler
        .base
        .syscalls_usage
        .clone();

    let call_info = finalize_execution(
        runner,
        cheatable_runtime.extended_runtime.hint_handler,
        n_total_args,
        program_extra_data_length,
        tracked_resource,
    )?;

    if call_info.execution.failed {
        // fallback to the last pc in the trace if user did not set `panic-backtrace = true` in `Scarb.toml`
        let pcs = if let Some(panic_traceback) = cheatable_runtime.extended_runtime.panic_traceback
        {
            panic_traceback
        } else {
            trace
                .last()
                .map(|last| vec![last.pc])
                .expect("trace should have at least one entry")
        };
        cheatable_runtime
            .extension
            .cheatnet_state
            .register_error(class_hash, pcs);
    }

    let (vm_trace, vm_memory) = vm_artifacts(trace, memory, cheatnet_state);

    // TODO(#4250): Investigate if we can simplify our logic given that syscall usage is now present in `CallInfo`
    let (syscall_usage_vm_resources, syscall_usage_sierra_gas) = match tracked_resource {
        TrackedResource::CairoSteps => (syscall_usage, SyscallUsageMap::default()),
        TrackedResource::SierraGas => (SyscallUsageMap::default(), syscall_usage),
    };

    Ok(CallInfoWithExecutionData {
        call_info,
        syscall_usage_vm_resources,
        syscall_usage_sierra_gas,
        vm_trace,
        vm_memory,
    })
    // endregion
}

/// VM memory to carry out of a call. Only the starkloupe fork collects it.
#[cfg(feature = "starkloupe")]
// The `Option` keeps one signature across both builds; without the feature there
// is nothing to return.
#[expect(clippy::unnecessary_wraps)]
fn relocated_memory(runner: &CairoRunner) -> Option<Vec<Option<Felt>>> {
    Some(runner.relocated_memory.clone())
}

#[cfg(not(feature = "starkloupe"))]
fn relocated_memory(_runner: &CairoRunner) -> Option<Vec<Option<Felt>>> {
    None
}

/// Decides what happens to the VM artifacts of a finished call.
///
/// Upstream attaches the trace to the current call and carries nothing further.
/// The starkloupe fork carries both out so they can also be attached to calls
/// that failed, which the caller cannot do once the trace has been consumed.
#[cfg(feature = "starkloupe")]
fn vm_artifacts(
    trace: Vec<RelocatedTraceEntry>,
    memory: Option<Vec<Option<Felt>>>,
    _cheatnet_state: &mut CheatnetState,
) -> (Option<Vec<RelocatedTraceEntry>>, Option<Vec<Option<Felt>>>) {
    (Some(trace), memory)
}

#[cfg(not(feature = "starkloupe"))]
fn vm_artifacts(
    trace: Vec<RelocatedTraceEntry>,
    _memory: Option<Vec<Option<Felt>>>,
    cheatnet_state: &mut CheatnetState,
) -> (Option<Vec<RelocatedTraceEntry>>, Option<Vec<Option<Felt>>>) {
    cheatnet_state
        .trace_data
        .set_vm_trace_for_current_call(trace);
    (None, None)
}

// crates/blockifier/src/execution/cairo1_execution.rs:236 (run_entry_point)
pub fn cheatable_run_entry_point(
    runner: &mut CairoRunner,
    hint_processor: &mut dyn HintProcessor,
    entry_point: &EntryPointV1,
    args: &Args,
    program_segment_size: usize,
) -> Result<(), EntryPointExecutionError> {
    // region: Modified blockifier code
    // Opposite to blockifier
    let verify_secure = false;
    // endregion
    let args: Vec<&CairoArg> = args.iter().collect();

    let result = runner
        .run_from_entrypoint(
            entry_point.pc(),
            &args,
            verify_secure,
            Some(program_segment_size),
            hint_processor,
        )
        .map_err(Box::new);

    // region: Modified blockifier code
    // Upstream propagates a run failure before relocating. The starkloupe fork
    // relocates first so a trace exists for the error path too, but the original
    // run failure still takes precedence over any relocation error.
    #[cfg(not(feature = "starkloupe"))]
    result?;

    // Relocate trace to then collect it
    let relocation_result = runner
        .relocate(true, true)
        .map_err(CairoRunError::from)
        .map_err(Box::new);

    #[cfg(feature = "starkloupe")]
    result?;

    relocation_result?;
    // endregion

    Ok(())
}
