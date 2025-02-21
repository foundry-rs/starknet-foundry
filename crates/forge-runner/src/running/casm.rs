use anyhow::Result;
use cairo_lang_casm::instructions::Instruction;
use cairo_lang_runner::{
    SierraCasmRunner,
    casm_run::{build_cairo_runner, run_function_with_runner},
    initialize_vm,
};
use cairo_vm::{
    hint_processor::hint_processor_definition::HintProcessor,
    serde::deserialize_program::HintParams,
    types::builtin_name::BuiltinName,
    types::relocatable::MaybeRelocatable,
    vm::{errors::cairo_run_errors::CairoRunError, runners::cairo_runner::CairoRunner},
};
use starknet_types_core::felt::Felt;
use universal_sierra_compiler_api::{
    AssembledCairoProgramWithSerde, AssembledProgramWithDebugInfo,
};

pub fn get_assembled_program(
    casm_program: &AssembledProgramWithDebugInfo,
    header: Vec<Instruction>,
) -> AssembledCairoProgramWithSerde {
    let mut assembled_program: AssembledCairoProgramWithSerde =
        casm_program.assembled_cairo_program.clone();

    add_header(header, &mut assembled_program);
    add_footer(&mut assembled_program);

    assembled_program
}

pub fn run_assembled_program(
    assembled_program: &AssembledCairoProgramWithSerde,
    builtins: Vec<BuiltinName>,
    hints_dict: std::collections::HashMap<usize, Vec<HintParams>>,
    hint_processor: &mut dyn HintProcessor,
) -> Result<CairoRunner, Box<CairoRunError>> {
    let data: Vec<MaybeRelocatable> = assembled_program
        .bytecode
        .iter()
        .map(Felt::from)
        .map(MaybeRelocatable::from)
        .collect();
    let data_len = data.len();

    let mut runner = build_cairo_runner(data, builtins, hints_dict)?;

    run_function_with_runner(data_len, initialize_vm, hint_processor, &mut runner)?;

    Ok(runner)
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

fn add_footer(assembled_program: &mut AssembledCairoProgramWithSerde) {
    let footer = SierraCasmRunner::create_code_footer();

    for instruction in footer {
        assembled_program
            .bytecode
            .extend(instruction.assemble().encode().into_iter());
    }
}
