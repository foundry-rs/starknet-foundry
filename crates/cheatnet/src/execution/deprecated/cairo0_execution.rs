use crate::execution::deprecated::syscalls::CheatableSyscallHandler;
use crate::state::CheatnetState;
use blockifier::execution::call_info::CallInfo;
use blockifier::execution::contract_class::ContractClassV0;
use blockifier::execution::deprecated_entry_point_execution::{
    finalize_execution, initialize_execution_context, prepare_call_arguments, VmExecutionContext,
};
use blockifier::execution::entry_point::{
    CallEntryPoint, EntryPointExecutionContext, EntryPointExecutionResult, ExecutionResources,
};
use blockifier::execution::errors::VirtualMachineExecutionError;
use blockifier::execution::execution_utils::Args;
use blockifier::state::state_api::State;
use cairo_vm::hint_processor::hint_processor_definition::HintProcessor;
use cairo_vm::vm::runners::cairo_runner::{CairoArg, CairoRunner};
use cairo_vm::vm::vm_core::VirtualMachine;

// blockifier/src/execution/deprecated_execution.rs:36 (execute_entry_point_call)
pub fn execute_entry_point_call_cairo0(
    call: CallEntryPoint,
    contract_class: ContractClassV0,
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
        entry_point_pc,
    } = initialize_execution_context(&call, contract_class, state, resources, context)?;

    let (implicit_args, args) = prepare_call_arguments(
        &call,
        &mut vm,
        initial_syscall_ptr,
        &mut syscall_handler.read_only_segments,
    )?;
    let n_total_args = args.len();

    // Fix the VM resources, in order to calculate the usage of this run at the end.
    let previous_vm_resources = syscall_handler.resources.vm_resources.clone();

    // region: Modified blockifier code
    let mut cheatable_syscall_handler = CheatableSyscallHandler {
        child: syscall_handler,
        cheatnet_state,
    };

    // Execute.
    cheatable_run_entry_point(
        &mut vm,
        &mut runner,
        &mut cheatable_syscall_handler,
        entry_point_pc,
        &args,
    )?;
    // endregion

    Ok(finalize_execution(
        vm,
        runner,
        cheatable_syscall_handler.child,
        call,
        previous_vm_resources,
        implicit_args,
        n_total_args,
    )?)
}

// blockifier/src/execution/deprecated_execution.rs:192 (run_entry_point)
pub fn cheatable_run_entry_point(
    vm: &mut VirtualMachine,
    runner: &mut CairoRunner,
    hint_processor: &mut dyn HintProcessor,
    entry_point_pc: usize,
    args: &Args,
) -> Result<(), VirtualMachineExecutionError> {
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
        vm,
        hint_processor,
    )?;

    Ok(())
}
