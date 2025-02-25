use crate::package_tests::TestDetails;
use cairo_lang_casm::instructions::Instruction;
use cairo_lang_runnable_utils::builder::{EntryCodeConfig, create_entry_code_from_params};
use cairo_vm::types::builtin_name::BuiltinName;
use universal_sierra_compiler_api::AssembledProgramWithDebugInfo;

// TODO(#2953)
pub fn create_entry_code(
    test_details: &TestDetails,
    casm_program: &AssembledProgramWithDebugInfo,
) -> (Vec<Instruction>, Vec<BuiltinName>) {
    let sierra_instruction_idx = test_details.sierra_entry_point_statement_idx;
    let casm_entry_point_offset = casm_program.debug_info[sierra_instruction_idx].0;

    create_entry_code_from_params(
        &test_details.parameter_types,
        &test_details.return_types,
        casm_entry_point_offset,
        EntryCodeConfig::testing(),
    )
    .unwrap()
}
