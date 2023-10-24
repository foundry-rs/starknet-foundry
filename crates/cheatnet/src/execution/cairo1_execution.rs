use super::cheatable_syscall_handler::CheatableSyscallHandler;
use crate::execution::contract_execution_syscall_handler::ContractExecutionSyscallHandler;
use crate::state::CheatnetState;
use blockifier::execution::call_info::CallInfo;
use blockifier::{
    execution::{
        cairo1_execution::{
            finalize_execution, initialize_execution_context, prepare_call_arguments,
            VmExecutionContext,
        },
        contract_class::{ContractClassV1, EntryPointV1},
        entry_point::{
            CallEntryPoint, EntryPointExecutionContext, EntryPointExecutionResult,
            ExecutionResources,
        },
        errors::{EntryPointExecutionError, VirtualMachineExecutionError},
        execution_utils::Args,
    },
    state::state_api::State,
};
use cairo_vm::{
    hint_processor::hint_processor_definition::HintProcessor,
    vm::{
        runners::cairo_runner::{CairoArg, CairoRunner},
        vm_core::VirtualMachine,
    },
};

// blockifier/src/execution/cairo1_execution.rs:48 (execute_entry_point_call)
pub fn execute_entry_point_call_cairo1(
    call: CallEntryPoint,
    contract_class: &ContractClassV1,
    state: &mut dyn State,
    cheatnet_state: &mut CheatnetState, // Added parameter
    resources: &mut ExecutionResources,
    context: &mut EntryPointExecutionContext,
) -> EntryPointExecutionResult<CallInfo> {
    let VmExecutionContext {
        mut runner,
        mut vm,
        mut syscall_handler,
        initial_syscall_ptr,
        entry_point,
        program_extra_data_length,
    } = initialize_execution_context(call, contract_class, state, resources, context)?;

    let args = prepare_call_arguments(
        &syscall_handler.call,
        &mut vm,
        initial_syscall_ptr,
        &mut syscall_handler.read_only_segments,
        &entry_point,
    )?;
    let n_total_args = args.len();

    // Snapshot the VM resources, in order to calculate the usage of this run at the end.
    let previous_vm_resources = syscall_handler.resources.vm_resources.clone();

    // region: Modified blockifier code
    let mut cheatable_syscall_handler = CheatableSyscallHandler {
        syscall_handler,
        cheatnet_state,
    };
    let mut contract_execution_syscall_handler =
        ContractExecutionSyscallHandler::wrap(&mut cheatable_syscall_handler);

    // Execute.
    cheatable_run_entry_point(
        &mut vm,
        &mut runner,
        &mut contract_execution_syscall_handler,
        &entry_point,
        &args,
        program_extra_data_length,
    )?;
    // endregion

    let call_info = finalize_execution(
        vm,
        runner,
        cheatable_syscall_handler.syscall_handler,
        previous_vm_resources,
        n_total_args,
        program_extra_data_length,
    )?;
    if call_info.execution.failed {
        return Err(EntryPointExecutionError::ExecutionFailed {
            error_data: call_info.execution.retdata.0,
        });
    }

    Ok(call_info)
}

// crates/blockifier/src/execution/cairo1_execution.rs:236 (run_entry_point)
pub fn cheatable_run_entry_point(
    vm: &mut VirtualMachine,
    runner: &mut CairoRunner,
    hint_processor: &mut dyn HintProcessor,
    entry_point: &EntryPointV1,
    args: &Args,
    program_segment_size: usize,
) -> Result<(), VirtualMachineExecutionError> {
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
        vm,
        hint_processor,
    )?;

    Ok(())
}
