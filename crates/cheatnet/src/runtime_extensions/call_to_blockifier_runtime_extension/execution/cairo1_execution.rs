use crate::runtime_extensions::call_to_blockifier_runtime_extension::execution::entry_point::{
    ContractClassEntryPointExecutionResult, EntryPointExecutionErrorWithTrace, OnErrorLastPc,
};
use crate::runtime_extensions::call_to_blockifier_runtime_extension::CheatnetState;
use crate::runtime_extensions::cheatable_starknet_runtime_extension::CheatableStarknetRuntimeExtension;
use crate::runtime_extensions::common::get_relocated_vm_trace;
use blockifier::execution::entry_point_execution::{
    finalize_execution, initialize_execution_context, prepare_call_arguments, VmExecutionContext,
};
use blockifier::{
    execution::{
        contract_class::{ContractClassV1, EntryPointV1},
        entry_point::{CallEntryPoint, EntryPointExecutionContext},
        errors::EntryPointExecutionError,
        execution_utils::Args,
    },
    state::state_api::State,
};
use cairo_vm::vm::errors::cairo_run_errors::CairoRunError;
use cairo_vm::{
    hint_processor::hint_processor_definition::HintProcessor,
    vm::runners::cairo_runner::{CairoArg, CairoRunner, ExecutionResources},
};
use runtime::{ExtendedRuntime, StarknetRuntime};

// blockifier/src/execution/cairo1_execution.rs:48 (execute_entry_point_call)
pub fn execute_entry_point_call_cairo1(
    call: CallEntryPoint,
    contract_class: &ContractClassV1,
    state: &mut dyn State,
    cheatnet_state: &mut CheatnetState, // Added parameter
    resources: &mut ExecutionResources,
    context: &mut EntryPointExecutionContext,
) -> ContractClassEntryPointExecutionResult {
    let VmExecutionContext {
        mut runner,
        mut syscall_handler,
        initial_syscall_ptr,
        entry_point,
        program_extra_data_length,
    } = initialize_execution_context(call, contract_class, state, resources, context)?;

    let args = prepare_call_arguments(
        &syscall_handler.base.call,
        &mut runner,
        initial_syscall_ptr,
        &mut syscall_handler.read_only_segments,
        &entry_point,
    )?;
    let n_total_args = args.len();

    // Snapshot the VM resources, in order to calculate the usage of this run at the end.
    let previous_vm_resources = syscall_handler.resources.clone();

    // region: Modified blockifier code

    let mut cheatable_runtime = ExtendedRuntime {
        extension: CheatableStarknetRuntimeExtension { cheatnet_state },
        extended_runtime: StarknetRuntime {
            hint_handler: syscall_handler,
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
    .on_error_get_last_pc(&mut runner)?;

    let trace = get_relocated_vm_trace(&mut runner);

    let syscall_counter = cheatable_runtime
        .extended_runtime
        .hint_handler
        .syscall_counter
        .clone();

    let call_info = finalize_execution(
        runner,
        cheatable_runtime.extended_runtime.hint_handler,
        previous_vm_resources,
        n_total_args,
        program_extra_data_length,
    )?;
    if call_info.execution.failed {
        return Err(EntryPointExecutionErrorWithTrace {
            source: EntryPointExecutionError::ExecutionFailed {
                error_data: call_info.execution.retdata.0,
            },
            trace,
        });
    }

    Ok((call_info, syscall_counter, trace))
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
