use crate::{
    function_args,
    package_tests::{
        raw::TestTargetRaw,
        with_config::{TestCaseWithConfig, TestTargetWithConfig},
        TestDetails,
    },
    running::config_run::run_config_pass,
};
use anyhow::{anyhow, Result};
use cairo_lang_sierra::{
    extensions::core::{CoreLibfunc, CoreType},
    ids::ConcreteTypeId,
    program::{GenFunction, StatementIdx, TypeDeclaration},
    program_registry::ProgramRegistry,
};
use cairo_lang_sierra_type_size::get_type_size_map;
use cairo_lang_utils::unordered_hash_map::UnorderedHashMap;
use starknet_types_core::felt::Felt;
use std::{collections::HashMap, sync::Arc};
use universal_sierra_compiler_api::{compile_sierra_at_path, SierraType};

pub fn test_target_with_config(test_target_raw: TestTargetRaw) -> Result<TestTargetWithConfig> {
    macro_rules! by_id {
        ($field:ident) => {{
            let temp: HashMap<_, _> = test_target_raw
                .sierra_program
                .program
                .$field
                .iter()
                .map(|f| (f.id.id, f))
                .collect();

            temp
        }};
    }
    let funcs = by_id!(funcs);
    let type_declarations = by_id!(type_declarations);

    let casm_program = Arc::new(
        compile_sierra_at_path(&test_target_raw.sierra_file_path, &SierraType::Raw)?.into(),
    );

    let sierra_program_registry =
        ProgramRegistry::<CoreType, CoreLibfunc>::new(&test_target_raw.sierra_program.program)?;
    let type_size_map = get_type_size_map(
        &test_target_raw.sierra_program.program,
        &sierra_program_registry,
    )
    .ok_or_else(|| anyhow!("can not get type size map"))?;

    let default_executables = vec![];
    let debug_info = test_target_raw.sierra_program.debug_info.clone();
    let executables = debug_info
        .as_ref()
        .and_then(|info| info.executables.get("snforge_internal_test_executable"))
        .unwrap_or(&default_executables);

    let test_cases = executables
        .iter()
        .map(|case| -> Result<TestCaseWithConfig> {
            let func = funcs[&case.id];

            let test_details = build_test_details(func, &type_declarations, &type_size_map);
            let args = prepare_args(func, &type_declarations);

            let raw_config = run_config_pass(args, &test_details, &casm_program)?;

            Ok(TestCaseWithConfig {
                config: raw_config.into(),
                name: case.debug_name.clone().unwrap().into(),
                test_details,
            })
        })
        .collect::<Result<_>>()?;

    Ok(TestTargetWithConfig {
        tests_location: test_target_raw.tests_location,
        test_cases,
        sierra_program: test_target_raw.sierra_program,
        casm_program,
    })
}

fn build_test_details(
    func: &GenFunction<StatementIdx>,
    type_declarations: &HashMap<u64, &TypeDeclaration>,
    type_size_map: &UnorderedHashMap<ConcreteTypeId, i16>,
) -> TestDetails {
    let map_types = |concrete_types: &[ConcreteTypeId]| {
        concrete_types
            .iter()
            .map(|ty| {
                let ty = type_declarations[&ty.id];

                (ty.long_id.generic_id.clone(), type_size_map[&ty.id])
            })
            .collect()
    };

    TestDetails {
        sierra_entry_point_statement_idx: func.entry_point.0,
        parameter_types: map_types(&func.signature.param_types),
        return_types: map_types(&func.signature.ret_types),
    }
}

fn prepare_args(
    func: &GenFunction<StatementIdx>,
    type_declarations: &HashMap<u64, &TypeDeclaration>,
) -> Vec<Felt> {
    let args = function_args(func, type_declarations);

    // trick to fix current fuzzer,
    // it supports only u256 from types bigger than 1 felt
    // since we have arguments count we need to know how many u256
    // are there and add this to length, this way we got
    // correct unmber of felts, this
    // should be removed with new fuzzer logic so it is not extensible
    let u256_occurrences = args
        .iter()
        .filter(|arg| {
            arg.generic_id.0 == "Struct"
                && matches!(
                    arg.generic_args.first(),
                    Some(cairo_lang_sierra::program::GenericArg::UserType(
                        cairo_lang_sierra::ids::UserTypeId {
                            debug_name: Some(name),
                            ..
                        }
                    )) if name == "core::integer::u256"
                )
        })
        .count();

    vec![Felt::from(0_u8); args.len() + u256_occurrences]
}
