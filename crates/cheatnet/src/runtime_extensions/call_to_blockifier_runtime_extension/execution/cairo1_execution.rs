use crate::runtime_extensions::call_to_blockifier_runtime_extension::CheatnetState;
use crate::runtime_extensions::call_to_blockifier_runtime_extension::execution::entry_point::{
    ContractClassEntryPointExecutionResult, EntryPointExecutionErrorWithTrace,
    extract_trace_and_register_errors,
};
use crate::runtime_extensions::cheatable_starknet_runtime_extension::CheatableStarknetRuntimeExtension;
use crate::runtime_extensions::common::get_relocated_vm_trace;
use blockifier::execution::contract_class::CompiledClassV1;
use blockifier::execution::entry_point::ExecutableCallEntryPoint;
use blockifier::execution::entry_point_execution::{
    ExecutionRunnerMode, VmExecutionContext, finalize_execution,
    initialize_execution_context_with_runner_mode, prepare_call_arguments,
};
use blockifier::execution::stack_trace::{
    Cairo1RevertHeader, extract_trailing_cairo1_revert_trace,
};
use blockifier::{
    execution::{
        contract_class::EntryPointV1, entry_point::EntryPointExecutionContext,
        errors::EntryPointExecutionError, execution_utils::Args,
    },
    state::state_api::State,
};
use cairo_vm::vm::errors::cairo_run_errors::CairoRunError;
use cairo_vm::{
    hint_processor::hint_processor_definition::HintProcessor,
    vm::runners::cairo_runner::{CairoArg, CairoRunner},
};
use runtime::{ExtendedRuntime, StarknetRuntime};

// blockifier/src/execution/cairo1_execution.rs:48 (execute_entry_point_call)
pub fn execute_entry_point_call_cairo1(
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
            user_args: vec![],
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
        extract_trace_and_register_errors(
            source,
            class_hash,
            &mut runner,
            cheatable_runtime.extension.cheatnet_state,
        )
    })?;

    let trace = get_relocated_vm_trace(&mut runner);
    let syscall_usage_map = cheatable_runtime
        .extended_runtime
        .hint_handler
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

        return Err(EntryPointExecutionErrorWithTrace {
            source: EntryPointExecutionError::ExecutionFailed {
                error_trace: extract_trailing_cairo1_revert_trace(
                    &call_info,
                    Cairo1RevertHeader::Execution,
                ),
            },
            trace: Some(trace),
        });
    }

    Ok((call_info, syscall_usage_map, Some(trace)))
    // endregion
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

    runner.run_from_entrypoint(
        entry_point.pc(),
        &args,
        verify_secure,
        Some(program_segment_size),
        hint_processor,
    )?;

    // region: Modified blockifier code
    // Relocate trace to then collect it
    runner.relocate(true).map_err(CairoRunError::from)?;
    // endregion

    Ok(())
}
