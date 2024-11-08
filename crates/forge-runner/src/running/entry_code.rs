use crate::package_tests::TestDetails;
use cairo_lang_casm::instructions::Instruction;
use cairo_lang_runner::{Arg, SierraCasmRunner};
use cairo_vm::types::builtin_name::BuiltinName;
use starknet_types_core::felt::Felt;
use universal_sierra_compiler_api::AssembledProgramWithDebugInfo;

pub fn create_entry_code(
    args: Vec<Felt>,
    test_details: &TestDetails,
    casm_program: &AssembledProgramWithDebugInfo,
) -> (Vec<Instruction>, Vec<BuiltinName>) {
    let initial_gas = usize::MAX;
    let runner_args: Vec<Arg> = args.into_iter().map(Arg::Value).collect();
    let sierra_instruction_idx = test_details.sierra_entry_point_statement_idx;
    let casm_entry_point_offset = casm_program.debug_info[sierra_instruction_idx].0;

    SierraCasmRunner::create_entry_code_from_params(
        &test_details.parameter_types,
        &runner_args,
        initial_gas,
        casm_entry_point_offset,
    )
    .unwrap()
}
