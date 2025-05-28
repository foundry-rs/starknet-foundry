use crate::package_tests::TestDetails;
use crate::running::copied_code::prepare_program_extra_data;
use blockifier::execution::contract_class::EntryPointV1;
use blockifier::execution::entry_point::{EntryPointExecutionContext, ExecutableCallEntryPoint};
use blockifier::execution::errors::PreExecutionError;
use blockifier::execution::execution_utils::ReadOnlySegments;
use blockifier::execution::syscalls::hint_processor::SyscallHintProcessor;
use blockifier::state::state_api::State;
use cairo_lang_casm::hints::Hint;
use cairo_vm::types::builtin_name::BuiltinName;
use cairo_vm::types::layout_name::LayoutName;
use cairo_vm::types::program::Program;
use cairo_vm::types::relocatable::Relocatable;
use cairo_vm::vm::runners::cairo_runner::CairoRunner;
use cheatnet::constants::build_test_entry_point;
use starknet_api::deprecated_contract_class::EntryPointOffset;
use std::collections::HashMap;
use universal_sierra_compiler_api::AssembledProgramWithDebugInfo;

// Based on structure from https://github.com/starkware-libs/sequencer/blob/e417a9e7d50cbd78065d357763df2fbc2ad41f7c/crates/blockifier/src/execution/entry_point_execution.rs#L39
// Logic of `initialize_execution_context` had to be modified so this struct ended up modified as well.
// Probably won't be possible to upstream it.
pub struct VmExecutionContext<'a> {
    pub runner: CairoRunner,
    pub syscall_handler: SyscallHintProcessor<'a>,
    pub initial_syscall_ptr: Relocatable,
    // Additional data required for execution is appended after the program bytecode.
    pub program_extra_data_length: usize,
}

// Based on code from https://github.com/starkware-libs/sequencer/blob/e417a9e7d50cbd78065d357763df2fbc2ad41f7c/crates/blockifier/src/execution/entry_point_execution.rs#L122
// Enough of the logic of this had to be changed that probably it won't be possible to upstream it
pub fn initialize_execution_context<'a>(
    call: ExecutableCallEntryPoint,
    hints: &'a HashMap<String, Hint>,
    program: &Program,
    state: &'a mut dyn State,
    context: &'a mut EntryPointExecutionContext,
) -> Result<VmExecutionContext<'a>, PreExecutionError> {
    // Instantiate Cairo runner.
    let proof_mode = false;
    let trace_enabled = true;
    let mut runner = CairoRunner::new(
        program,
        LayoutName::all_cairo,
        // TODO: Pass real dynamic layout params
        None,
        proof_mode,
        trace_enabled,
        // TODO: Pass real disable trace padding
        false,
    )?;

    runner.initialize_function_runner_cairo_1(&builtins_from_program(program))?;
    let mut read_only_segments = ReadOnlySegments::default();
    let program_extra_data_length = prepare_program_extra_data(
        &mut runner,
        program.data_len(),
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
        hints,
        read_only_segments,
    );

    Ok(VmExecutionContext {
        runner,
        syscall_handler,
        initial_syscall_ptr,
        program_extra_data_length,
    })
}

// Builtins field is private in program, so we need this workaround
fn builtins_from_program(program: &Program) -> Vec<BuiltinName> {
    program.iter_builtins().copied().collect::<Vec<_>>()
}

pub fn entry_point_initial_budget(syscall_hint_processor: &SyscallHintProcessor) -> u64 {
    syscall_hint_processor
        .base
        .context
        .gas_costs()
        .base
        .entry_point_initial_budget
}

pub fn build_test_call_and_entry_point(
    test_details: &TestDetails,
    casm_program: &AssembledProgramWithDebugInfo,
    program: &Program,
) -> (ExecutableCallEntryPoint, EntryPointV1) {
    let sierra_instruction_idx = test_details.sierra_entry_point_statement_idx;
    let casm_entry_point_offset = casm_program.debug_info[sierra_instruction_idx].0;

    let call = build_test_entry_point();
    let entry_point = EntryPointV1 {
        selector: call.entry_point_selector,
        offset: EntryPointOffset(casm_entry_point_offset),
        builtins: builtins_from_program(program),
    };
    (call, entry_point)
}
