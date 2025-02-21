use crate::runtime_extensions::call_to_blockifier_runtime_extension::CheatnetState;
use crate::runtime_extensions::call_to_blockifier_runtime_extension::execution::entry_point::{
    ContractClassEntryPointExecutionResult, OnErrorLastPc,
};
use crate::runtime_extensions::deprecated_cheatable_starknet_extension::DeprecatedCheatableStarknetRuntimeExtension;
use crate::runtime_extensions::deprecated_cheatable_starknet_extension::runtime::{
    DeprecatedExtendedRuntime, DeprecatedStarknetRuntime,
};
use blockifier::execution::contract_class::ContractClassV0;
use blockifier::execution::deprecated_entry_point_execution::{
    VmExecutionContext, finalize_execution, initialize_execution_context, prepare_call_arguments,
};
use blockifier::execution::entry_point::{CallEntryPoint, EntryPointExecutionContext};
use blockifier::execution::errors::EntryPointExecutionError;
use blockifier::execution::execution_utils::Args;
use blockifier::state::state_api::State;
use cairo_vm::hint_processor::hint_processor_definition::HintProcessor;
use cairo_vm::vm::runners::cairo_runner::{CairoArg, CairoRunner, ExecutionResources};

// blockifier/src/execution/deprecated_execution.rs:36 (execute_entry_point_call)
pub fn execute_entry_point_call_cairo0(
    call: CallEntryPoint,
    contract_class: ContractClassV0,
    state: &mut dyn State,
    cheatnet_state: &mut CheatnetState,
    resources: &mut ExecutionResources,
    context: &mut EntryPointExecutionContext,
) -> ContractClassEntryPointExecutionResult {
    let VmExecutionContext {
        mut runner,
        mut syscall_handler,
        initial_syscall_ptr,
        entry_point_pc,
    } = initialize_execution_context(&call, contract_class, state, resources, context)?;

    let (implicit_args, args) = prepare_call_arguments(
        &call,
        &mut runner,
        initial_syscall_ptr,
        &mut syscall_handler.read_only_segments,
    )?;
    let n_total_args = args.len();

    // Fix the VM resources, in order to calculate the usage of this run at the end.
    let previous_vm_resources = syscall_handler.resources.clone();

    // region: Modified blockifier code
    let cheatable_extension = DeprecatedCheatableStarknetRuntimeExtension { cheatnet_state };
    let mut cheatable_syscall_handler = DeprecatedExtendedRuntime {
        extension: cheatable_extension,
        extended_runtime: DeprecatedStarknetRuntime {
            hint_handler: syscall_handler,
        },
    };

    // Execute.
    cheatable_run_entry_point(
        &mut runner,
        &mut cheatable_syscall_handler,
        entry_point_pc,
        &args,
    )
    .on_error_get_last_pc(&mut runner)?;

    let syscall_counter = cheatable_syscall_handler
        .extended_runtime
        .hint_handler
        .syscall_counter
        .clone();

    let execution_result = finalize_execution(
        runner,
        cheatable_syscall_handler.extended_runtime.hint_handler,
        call,
        previous_vm_resources,
        implicit_args,
        n_total_args,
    )?;

    Ok((execution_result, syscall_counter, None))
    // endregion
}

// blockifier/src/execution/deprecated_execution.rs:192 (run_entry_point)
pub fn cheatable_run_entry_point(
    runner: &mut CairoRunner,
    hint_processor: &mut dyn HintProcessor,
    entry_point_pc: usize,
    args: &Args,
) -> Result<(), EntryPointExecutionError> {
    // region: Modified blockifier code
    // Opposite to blockifier
    let verify_secure = false;
    // endregion
    let program_segment_size = None; // Infer size from program.
    let args: Vec<&CairoArg> = args.iter().collect();

    runner.run_from_entrypoint(
        entry_point_pc,
        &args,
        verify_secure,
        program_segment_size,
        hint_processor,
    )?;

    Ok(())
}
