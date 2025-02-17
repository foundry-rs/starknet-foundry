use crate::runtime_extensions::call_to_blockifier_runtime_extension::execution::entry_point::{
    ContractClassEntryPointExecutionResult, EntryPointExecutionErrorWithTrace, OnErrorLastPc,
};
use crate::runtime_extensions::call_to_blockifier_runtime_extension::CheatnetState;
use crate::runtime_extensions::cheatable_starknet_runtime_extension::CheatableStarknetRuntimeExtension;
use crate::runtime_extensions::common::get_relocated_vm_trace;
use blockifier::execution::contract_class::CompiledClassV1;
use blockifier::execution::entry_point_execution::{
    finalize_execution, prepare_call_arguments, VmExecutionContext,
};
use blockifier::execution::errors::PreExecutionError;
use blockifier::execution::execution_utils::{
    write_felt, write_maybe_relocatable, ReadOnlySegments,
};
use blockifier::execution::stack_trace::{
    extract_trailing_cairo1_revert_trace, Cairo1RevertHeader,
};
use blockifier::execution::syscalls::hint_processor::SyscallHintProcessor;
use blockifier::versioned_constants::GasCosts;
use blockifier::{
    execution::{
        contract_class::EntryPointV1,
        entry_point::{CallEntryPoint, EntryPointExecutionContext},
        errors::EntryPointExecutionError,
        execution_utils::Args,
    },
    state::state_api::State,
};
use cairo_vm::types::layout_name::LayoutName;
use cairo_vm::types::relocatable::MaybeRelocatable;
use cairo_vm::vm::errors::cairo_run_errors::CairoRunError;
use cairo_vm::{
    hint_processor::hint_processor_definition::HintProcessor,
    vm::runners::cairo_runner::{CairoArg, CairoRunner},
};
use runtime::{ExtendedRuntime, StarknetRuntime};
use starknet_types_core::felt::Felt;

// FIXME copied code
fn prepare_program_extra_data(
    runner: &mut CairoRunner,
    contract_class: &CompiledClassV1,
    read_only_segments: &mut ReadOnlySegments,
    gas_costs: &GasCosts,
) -> Result<usize, PreExecutionError> {
    // Create the builtin cost segment, the builtin order should be the same as the price builtin
    // array in the os in compiled_class.cairo in load_compiled_class_facts.
    let builtin_price_array = [
        gas_costs.builtins.pedersen,
        gas_costs.builtins.bitwise,
        gas_costs.builtins.ecop,
        gas_costs.builtins.poseidon,
        gas_costs.builtins.add_mod,
        gas_costs.builtins.mul_mod,
    ];

    let data = builtin_price_array
        .iter()
        .map(|&x| MaybeRelocatable::from(Felt::from(x)))
        .collect::<Vec<_>>();
    let builtin_cost_segment_start = read_only_segments.allocate(&mut runner.vm, &data)?;

    // Put a pointer to the builtin cost segment at the end of the program (after the
    // additional `ret` statement).
    let mut ptr = (runner.vm.get_pc() + contract_class.bytecode_length())?;
    // Push a `ret` opcode.

    write_felt(
        &mut runner.vm,
        &mut ptr,
        Felt::from(0x208b_7fff_7fff_7ffe_u128),
    )?;
    // Push a pointer to the builtin cost segment.
    write_maybe_relocatable(&mut runner.vm, &mut ptr, builtin_cost_segment_start)?;

    let program_extra_data_length = 2;
    Ok(program_extra_data_length)
}

// FIXME copied code
fn initialize_execution_context<'a>(
    call: CallEntryPoint,
    compiled_class: &'a CompiledClassV1,
    state: &'a mut dyn State,
    context: &'a mut EntryPointExecutionContext,
) -> Result<VmExecutionContext<'a>, PreExecutionError> {
    let entry_point = compiled_class.get_entry_point(&call)?;

    // Instantiate Cairo runner.
    let proof_mode = false;
    // FIXME modified blockifier code to enable traces
    let trace_enabled = true;
    let mut runner = CairoRunner::new(
        &compiled_class.0.program,
        LayoutName::starknet,
        proof_mode,
        trace_enabled,
    )?;

    runner.initialize_function_runner_cairo_1(&entry_point.builtins)?;
    let mut read_only_segments = ReadOnlySegments::default();
    let program_extra_data_length = prepare_program_extra_data(
        &mut runner,
        compiled_class,
        &mut read_only_segments,
        &context.versioned_constants().os_constants.gas_costs,
    )?;

    // Instantiate syscall handler.
    let initial_syscall_ptr = runner.vm.add_memory_segment();
    let syscall_handler = SyscallHintProcessor::new(
        state,
        context,
        initial_syscall_ptr,
        call,
        &compiled_class.hints,
        read_only_segments,
    );

    Ok(VmExecutionContext {
        runner,
        syscall_handler,
        initial_syscall_ptr,
        entry_point,
        program_extra_data_length,
    })
}

// blockifier/src/execution/cairo1_execution.rs:48 (execute_entry_point_call)
pub fn execute_entry_point_call_cairo1(
    call: CallEntryPoint,
    compiled_class_v1: &CompiledClassV1,
    state: &mut dyn State,
    cheatnet_state: &mut CheatnetState, // Added parameter
    // resources: &mut ExecutionResources,
    context: &mut EntryPointExecutionContext,
) -> ContractClassEntryPointExecutionResult {
    let tracked_resource = *context
        .tracked_resource_stack
        .last()
        .expect("Unexpected empty tracked resource.");
    let entry_point_initial_budget = context.gas_costs().base.entry_point_initial_budget;

    let VmExecutionContext {
        mut runner,
        mut syscall_handler,
        initial_syscall_ptr,
        entry_point,
        program_extra_data_length,
    } = initialize_execution_context(call, compiled_class_v1, state, context)?;

    let args = prepare_call_arguments(
        &syscall_handler.base.call,
        &mut runner,
        initial_syscall_ptr,
        &mut syscall_handler.read_only_segments,
        &entry_point,
        entry_point_initial_budget,
    )?;
    let n_total_args = args.len();

    // // Snapshot the VM resources, in order to calculate the usage of this run at the end.
    // let previous_vm_resources = syscall_handler.resources.clone();

    // region: Modified blockifier code

    let mut cheatable_runtime = ExtendedRuntime {
        extension: CheatableStarknetRuntimeExtension { cheatnet_state },
        extended_runtime: StarknetRuntime {
            hint_handler: syscall_handler,
            // FIXME use correct value
            user_args: vec![],
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
        n_total_args,
        program_extra_data_length,
        tracked_resource,
    )?;
    if call_info.execution.failed {
        return Err(EntryPointExecutionErrorWithTrace {
            source: EntryPointExecutionError::ExecutionFailed {
                error_trace: extract_trailing_cairo1_revert_trace(
                    &call_info,
                    Cairo1RevertHeader::Execution,
                ),
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
