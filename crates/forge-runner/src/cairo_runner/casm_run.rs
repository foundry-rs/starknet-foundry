use cairo_felt::Felt252;
use cairo_lang_casm::instructions::Instruction;
use cairo_lang_runner::casm_run::RunFunctionContext;
use cairo_vm::hint_processor::hint_processor_definition::HintProcessor;
use cairo_vm::serde::deserialize_program::{BuiltinName, HintParams, ReferenceManager};
use cairo_vm::types::program::Program;
use cairo_vm::types::relocatable::MaybeRelocatable;
use cairo_vm::vm::errors::cairo_run_errors::CairoRunError;
use cairo_vm::vm::runners::cairo_runner::CairoRunner;
use cairo_vm::vm::vm_core::VirtualMachine;
use std::collections::HashMap;

#[allow(dead_code)]
type RunFunctionRes = (Vec<Option<Felt252>>, usize);

pub fn run_function_with_runner(
    vm: &mut VirtualMachine,
    data_len: usize,
    additional_initialization: fn(
        context: RunFunctionContext<'_>,
    ) -> Result<(), Box<CairoRunError>>,
    hint_processor: &mut dyn HintProcessor,
    runner: &mut CairoRunner,
) -> Result<(), Box<CairoRunError>> {
    let end = runner.initialize(vm).map_err(CairoRunError::from)?;

    additional_initialization(RunFunctionContext { vm, data_len })?;

    runner
        .run_until_pc(end, vm, hint_processor)
        .map_err(CairoRunError::from)?;
    runner
        .end_run(true, false, vm, hint_processor)
        .map_err(CairoRunError::from)?;
    runner.relocate(vm, true).map_err(CairoRunError::from)?;
    Ok(())
}

pub fn build_runner(
    data: Vec<MaybeRelocatable>,
    builtins: Vec<BuiltinName>,
    hints_dict: HashMap<usize, Vec<HintParams>>,
) -> Result<CairoRunner, Box<CairoRunError>> {
    let program = Program::new(
        builtins,
        data,
        Some(0),
        hints_dict,
        ReferenceManager {
            references: Vec::new(),
        },
        HashMap::new(),
        vec![],
        None,
    )
    .map_err(CairoRunError::from)?;
    CairoRunner::new(&program, "all_cairo", false)
        .map_err(CairoRunError::from)
        .map_err(Box::new)
}

#[allow(dead_code)]
/// Runs `program` on layout with prime, and returns the memory layout and ap value.
/// Allows injecting custom `HintProcessor`.
pub fn run_function<'a, 'b: 'a, Instructions>(
    vm: &mut VirtualMachine,
    instructions: Instructions,
    builtins: Vec<BuiltinName>,
    additional_initialization: fn(
        context: RunFunctionContext<'_>,
    ) -> Result<(), Box<CairoRunError>>,
    hint_processor: &mut dyn HintProcessor,
    hints_dict: HashMap<usize, Vec<HintParams>>,
) -> Result<RunFunctionRes, Box<CairoRunError>>
where
    Instructions: Iterator<Item = &'a Instruction> + Clone,
{
    let data: Vec<MaybeRelocatable> = instructions
        .flat_map(|inst| inst.assemble().encode())
        .map(Felt252::from)
        .map(MaybeRelocatable::from)
        .collect();

    let data_len = data.len();
    let mut runner = build_runner(data, builtins, hints_dict)?;

    run_function_with_runner(
        vm,
        data_len,
        additional_initialization,
        hint_processor,
        &mut runner,
    )?;
    Ok((
        runner.relocated_memory,
        vm.get_relocated_trace().unwrap().last().unwrap().ap,
    ))
}
