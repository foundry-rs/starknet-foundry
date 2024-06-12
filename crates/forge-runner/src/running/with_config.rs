use crate::{
    function_args,
    package_tests::{
        raw::TestTargetRaw,
        with_config::{TestCaseWithConfig, TestTargetWithConfig},
        TestDetails,
    },
    running::config_run::get_config_for_test_case,
};
use anyhow::{anyhow, Result};
use cairo_felt::Felt252;
use cairo_lang_sierra::{
    extensions::core::{CoreLibfunc, CoreType},
    ids::{ConcreteTypeId, GenericTypeId},
    program::TypeDeclaration,
    program_registry::ProgramRegistry,
};
use cairo_lang_sierra_type_size::get_type_size_map;
use cairo_lang_utils::unordered_hash_map::UnorderedHashMap;
use std::{collections::HashMap, sync::Arc};
use universal_sierra_compiler_api::compile_sierra_to_casm;

pub fn test_target_with_config(test_target_raw: TestTargetRaw) -> Result<TestTargetWithConfig> {
    let funcs: HashMap<_, _> = test_target_raw
        .sierra_program
        .program
        .funcs
        .iter()
        .map(|f| (f.id.id, f))
        .collect();

    let types: HashMap<_, _> = test_target_raw
        .sierra_program
        .program
        .type_declarations
        .iter()
        .map(|ty| (ty.id.id, ty))
        .collect();

    let casm_program = Arc::new(compile_sierra_to_casm(
        &test_target_raw.sierra_program.program,
    )?);

    let sierra_program_registry =
        ProgramRegistry::<CoreType, CoreLibfunc>::new(&test_target_raw.sierra_program.program)?;
    let type_size_map = get_type_size_map(
        &test_target_raw.sierra_program.program,
        &sierra_program_registry,
    )
    .ok_or_else(|| anyhow!("can not get type size map"))?;

    let default_executables = vec![];
    let executables = test_target_raw
        .sierra_program
        .debug_info
        .executables
        .get("snforge_internal_test_executable")
        .unwrap_or(&default_executables);

    Ok(TestTargetWithConfig {
        tests_location: test_target_raw.tests_location,
        test_cases: executables
            .iter()
            .map(|case| -> Result<TestCaseWithConfig> {
                fn map_types(
                    concrete_type: &[ConcreteTypeId],
                    types: &HashMap<u64, &TypeDeclaration>,
                    type_size_map: &UnorderedHashMap<ConcreteTypeId, i16>,
                ) -> Vec<(GenericTypeId, i16)> {
                    concrete_type
                        .iter()
                        .map(|ty| {
                            let ty = types[&ty.id];

                            (ty.long_id.generic_id.clone(), type_size_map[&ty.id])
                        })
                        .collect()
                }

                let func = funcs[&case.id];

                let test_details = TestDetails {
                    sierra_entry_point_statement_idx: func.entry_point.0,
                    parameter_types: map_types(&func.signature.param_types, &types, &type_size_map),
                    return_types: map_types(&func.signature.ret_types, &types, &type_size_map),
                };

                let args = function_args(func);
                let raw_config = get_config_for_test_case(
                    vec![Felt252::from(0_u8); args.len()],
                    &test_details,
                    &casm_program,
                )?;

                Ok(TestCaseWithConfig {
                    config: raw_config.into(),
                    name: case.debug_name.clone().unwrap().into(),
                    test_details,
                })
            })
            .collect::<Result<_>>()?,
        sierra_program: test_target_raw.sierra_program,
        casm_program,
    })
}