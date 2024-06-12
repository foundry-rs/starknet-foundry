use crate::package_tests::TestDetails;
use anyhow::Result;
use blockifier::{
    execution::{
        entry_point::EntryPointExecutionContext, execution_utils::ReadOnlySegments,
        syscalls::hint_processor::SyscallHintProcessor,
    },
    state::state_api::State,
};
use cairo_felt::Felt252;
use cairo_lang_casm::{hints::Hint, instructions::Instruction};
use cairo_lang_runner::casm_run::hint_to_hint_params;
use cairo_lang_runner::{
    casm_run::{build_cairo_runner, run_function_with_runner},
    initialize_vm, Arg, SierraCasmRunner,
};
use cairo_lang_sierra::extensions::NoGenericArgsGenericType;
use cairo_lang_sierra::{extensions::segment_arena::SegmentArenaType, ids::GenericTypeId};
use cairo_vm::{
    hint_processor::hint_processor_definition::HintProcessor,
    serde::deserialize_program::{BuiltinName, HintParams},
    types::relocatable::{MaybeRelocatable, Relocatable},
    vm::{
        errors::cairo_run_errors::CairoRunError,
        runners::cairo_runner::{CairoRunner, ExecutionResources},
        vm_core::VirtualMachine,
    },
};
use cheatnet::constants::build_test_entry_point;
use std::collections::HashMap;
use std::default::Default;
use universal_sierra_compiler_api::{
    AssembledCairoProgramWithSerde, AssembledProgramWithDebugInfo,
};

pub fn get_assembled_program(
    casm_program: &AssembledProgramWithDebugInfo,
    entry_code: Vec<Instruction>,
    footer: Vec<Instruction>,
) -> AssembledCairoProgramWithSerde {
    let mut assembled_program: AssembledCairoProgramWithSerde =
        casm_program.assembled_cairo_program.clone();

    add_header(entry_code, &mut assembled_program);
    add_footer(footer, &mut assembled_program);

    assembled_program
}

pub fn run_with_runner(
    assembled_program: &AssembledCairoProgramWithSerde,
    builtins: Vec<BuiltinName>,
    hints_dict: std::collections::HashMap<usize, Vec<HintParams>>,
    hint_processor: &mut dyn HintProcessor,
) -> Result<(VirtualMachine, CairoRunner), Box<CairoRunError>> {
    let mut vm = VirtualMachine::new(true);

    let data: Vec<MaybeRelocatable> = assembled_program
        .bytecode
        .iter()
        .map(Felt252::from)
        .map(MaybeRelocatable::from)
        .collect();
    let data_len = data.len();

    let mut runner = build_cairo_runner(data, builtins, hints_dict)?;

    run_function_with_runner(
        &mut vm,
        data_len,
        initialize_vm,
        hint_processor,
        &mut runner,
    )?;

    Ok((vm, runner))
}

pub fn create_entry_code(
    args: Vec<Felt252>,
    test_details: &TestDetails,
    casm_program: &AssembledProgramWithDebugInfo,
) -> (Vec<Instruction>, Vec<BuiltinName>) {
    let initial_gas = usize::MAX;
    let runner_args: Vec<Arg> = args.into_iter().map(Arg::Value).collect();
    let sierra_instruction_idx = test_details.sierra_entry_point_statement_idx;
    let casm_entry_point_offset = casm_program.debug_info[sierra_instruction_idx].0;

    let (entry_code, builtins) = SierraCasmRunner::create_entry_code_from_params(
        &test_details.parameter_types,
        &runner_args,
        initial_gas,
        casm_entry_point_offset,
    )
    .unwrap();

    (entry_code, builtins)
}

pub fn get_syscall_segment_index(test_param_types: &[(GenericTypeId, i16)]) -> isize {
    // Segment arena is allocated conditionally, so segment index is automatically moved (+2 segments)
    if test_param_types
        .iter()
        .any(|(ty, _)| ty == &SegmentArenaType::ID)
    {
        12
    } else {
        10
    }
}

pub fn build_syscall_handler<'a>(
    blockifier_state: &'a mut dyn State,
    string_to_hint: &'a HashMap<String, Hint>,
    execution_resources: &'a mut ExecutionResources,
    context: &'a mut EntryPointExecutionContext,
    syscall_segment_index: isize,
) -> SyscallHintProcessor<'a> {
    let entry_point = build_test_entry_point();

    SyscallHintProcessor::new(
        blockifier_state,
        execution_resources,
        context,
        Relocatable {
            segment_index: syscall_segment_index,
            offset: 0,
        },
        entry_point,
        string_to_hint,
        ReadOnlySegments::default(),
    )
}

fn add_header(
    entry_code: Vec<Instruction>,
    assembled_program: &mut AssembledCairoProgramWithSerde,
) {
    let mut new_bytecode = vec![];
    let mut new_hints = vec![];
    for instruction in entry_code {
        if !instruction.hints.is_empty() {
            new_hints.push((new_bytecode.len(), instruction.hints.clone()));
        }
        new_bytecode.extend(instruction.assemble().encode().into_iter());
    }

    let new_bytecode_len = new_bytecode.len();
    assembled_program.hints.iter_mut().for_each(|hint| {
        hint.0 += new_bytecode_len;
    });

    assembled_program.hints = [new_hints, assembled_program.hints.clone()].concat();
    assembled_program.bytecode = [new_bytecode, assembled_program.bytecode.clone()].concat();
}

fn add_footer(footer: Vec<Instruction>, assembled_program: &mut AssembledCairoProgramWithSerde) {
    for instruction in footer {
        assembled_program
            .bytecode
            .extend(instruction.assemble().encode().into_iter());
    }
}

pub fn create_hints_dict(
    assembled_program: &AssembledCairoProgramWithSerde,
) -> (HashMap<String, Hint>, HashMap<usize, Vec<HintParams>>) {
    let string_to_hint: HashMap<String, Hint> = assembled_program
        .hints
        .iter()
        .flat_map(|(_, hints)| hints.iter().cloned())
        .map(|hint| (hint.representing_string(), hint))
        .collect();
    let hints_dict = assembled_program
        .hints
        .iter()
        .map(|(offset, hints)| {
            (
                *offset,
                hints
                    .iter()
                    .map(hint_to_hint_params)
                    .collect::<Vec<HintParams>>(),
            )
        })
        .collect();

    (string_to_hint, hints_dict)
}
