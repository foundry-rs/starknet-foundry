use crate::runtime_extensions::call_to_blockifier_runtime_extension::CheatnetState;
use crate::runtime_extensions::call_to_blockifier_runtime_extension::execution::entry_point::{
    CallInfoWithExecutionData, ContractClassEntryPointExecutionResult,
    extract_trace_and_register_errors,
};
use crate::runtime_extensions::deprecated_cheatable_starknet_extension::DeprecatedCheatableStarknetRuntimeExtension;
use crate::runtime_extensions::deprecated_cheatable_starknet_extension::runtime::{
    DeprecatedExtendedRuntime, DeprecatedStarknetRuntime,
};
use blockifier::execution::contract_class::{CompiledClassV0, TrackedResource};
use blockifier::execution::deprecated_entry_point_execution::{
    VmExecutionContext, finalize_execution, initialize_execution_context, prepare_call_arguments,
};
use blockifier::execution::entry_point::{EntryPointExecutionContext, ExecutableCallEntryPoint};
use blockifier::execution::errors::EntryPointExecutionError;
use blockifier::execution::execution_utils::Args;
use blockifier::execution::syscalls::hint_processor::SyscallUsageMap;
use blockifier::state::state_api::State;
use cairo_vm::hint_processor::hint_processor_definition::HintProcessor;
use cairo_vm::vm::runners::cairo_runner::{CairoArg, CairoRunner};

// blockifier/src/execution/deprecated_execution.rs:36 (execute_entry_point_call)
pub(crate) fn execute_entry_point_call_cairo0(
    call: ExecutableCallEntryPoint,
    compiled_class_v0: CompiledClassV0,
    state: &mut dyn State,
    cheatnet_state: &mut CheatnetState,
    context: &mut EntryPointExecutionContext,
) -> ContractClassEntryPointExecutionResult {
    let VmExecutionContext {
        mut runner,
        mut syscall_handler,
        initial_syscall_ptr,
        entry_point_pc,
    } = initialize_execution_context(&call, compiled_class_v0, state, context)?;

    let (implicit_args, args) = prepare_call_arguments(
        &call,
        &mut runner,
        initial_syscall_ptr,
        &mut syscall_handler.read_only_segments,
    )?;
    let n_total_args = args.len();

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
    .map_err(|source| {
        extract_trace_and_register_errors(
            source,
            call.class_hash,
            &mut runner,
            cheatable_syscall_handler.extension.cheatnet_state,
        )
    })?;

    let syscall_usage = cheatable_syscall_handler
        .extended_runtime
        .hint_handler
        .syscalls_usage
        .clone();

    let execution_result = finalize_execution(
        runner,
        cheatable_syscall_handler.extended_runtime.hint_handler,
        call,
        implicit_args,
        n_total_args,
    )?;

    let mut syscall_usage_vm_resources = SyscallUsageMap::default();
    let mut syscall_usage_sierra_gas = SyscallUsageMap::default();

    match execution_result.tracked_resource {
        TrackedResource::CairoSteps => syscall_usage_vm_resources.clone_from(&syscall_usage),
        TrackedResource::SierraGas => syscall_usage_sierra_gas.clone_from(&syscall_usage),
    }

    Ok(CallInfoWithExecutionData {
        call_info: execution_result,
        syscall_usage_vm_resources,
        syscall_usage_sierra_gas,
        vm_trace: None,
    })
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
