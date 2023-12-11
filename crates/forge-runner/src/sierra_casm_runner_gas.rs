use blockifier::execution::syscalls::hint_processor::SyscallHintProcessor;
use cairo_felt::Felt252;
use cairo_lang_runner::casm_run::RunFunctionContext;
use cairo_vm::serde::deserialize_program::{BuiltinName, HintParams, ReferenceManager};
use cairo_vm::types::program::Program;
use cairo_vm::types::relocatable::MaybeRelocatable;
use cairo_vm::vm::errors::cairo_run_errors::CairoRunError;
use cairo_vm::vm::errors::vm_errors::VirtualMachineError;
use cairo_vm::vm::runners::cairo_runner::CairoRunner;
use cairo_vm::vm::vm_core::VirtualMachine;
use cheatnet::runtime_extensions::forge_runtime_extension::ForgeRuntime;
use std::collections::HashMap;

use cairo_lang_casm::instructions::Instruction;

// casm_run::run_function
pub fn run_function<'a, 'b: 'a, Instructions>(
    vm: &mut VirtualMachine,
    instructions: Instructions,
    builtins: Vec<BuiltinName>,
    additional_initialization: fn(
        context: RunFunctionContext<'_>,
    ) -> Result<(), Box<CairoRunError>>,
    runtime: &mut ForgeRuntime,
    hints_dict: HashMap<usize, Vec<HintParams>>,
) -> Result<(Vec<Option<Felt252>>, usize), Box<CairoRunError>>
where
    Instructions: Iterator<Item = &'a Instruction> + Clone,
{
    let data: Vec<MaybeRelocatable> = instructions
        .flat_map(|inst| inst.assemble().encode())
        .map(Felt252::from)
        .map(MaybeRelocatable::from)
        .collect();

    let data_len = data.len();
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
    let mut runner = CairoRunner::new(&program, "starknet", false)
        .map_err(CairoRunError::from)
        .map_err(Box::new)?;

    let end = runner.initialize(vm).map_err(CairoRunError::from)?;

    additional_initialization(RunFunctionContext { vm, data_len })?;

    runner
        .run_until_pc(end, vm, runtime)
        .map_err(CairoRunError::from)?;
    runner
        .end_run(true, false, vm, runtime)
        .map_err(CairoRunError::from)?;
    runner.relocate(vm, true).map_err(CairoRunError::from)?;

    // changed region
    finalize(
        vm,
        &runner,
        &mut runtime
            .extended_runtime
            .extended_runtime
            .extended_runtime
            .extended_runtime
            .hint_handler,
        0,
        2,
    );
    // end region

    Ok((
        runner.relocated_memory,
        vm.get_relocated_trace().unwrap().last().unwrap().ap,
    ))
}

// similar to `finalize_execution` from blockifier
fn finalize(
    vm: &mut VirtualMachine,
    runner: &CairoRunner,
    syscall_handler: &mut SyscallHintProcessor<'_>,
    n_total_args: usize,
    program_extra_data_length: usize,
) {
    let program_start_ptr = runner
        .program_base
        .expect("The `program_base` field should be initialized after running the entry point.");
    let program_end_ptr = (program_start_ptr + runner.get_program().data_len()).unwrap();
    vm.mark_address_range_as_accessed(program_end_ptr, program_extra_data_length)
        .unwrap();

    let initial_fp = runner
        .get_initial_fp()
        .expect("The `initial_fp` field should be initialized after running the entry point.");
    // When execution starts the stack holds the EP arguments + [ret_fp, ret_pc].
    let args_ptr = (initial_fp - (n_total_args + 2)).unwrap();
    vm.mark_address_range_as_accessed(args_ptr, n_total_args)
        .unwrap();
    syscall_handler
        .read_only_segments
        .mark_as_accessed(vm)
        .unwrap();

    let vm_resources_without_inner_calls = runner
        .get_execution_resources(vm)
        .map_err(VirtualMachineError::TracerError)
        .unwrap()
        .filter_unused_builtins();
    syscall_handler.resources.vm_resources += &vm_resources_without_inner_calls;
}
