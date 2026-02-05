use crate::{
    forge_config::ForgeTrackedResource,
    package_tests::{
        TestDetails,
        raw::TestTargetRaw,
        with_config::{TestCaseWithConfig, TestTargetWithConfig},
    },
    running::config_run::run_config_pass,
};
use anyhow::Result;
use cairo_lang_sierra::{
    ids::ConcreteTypeId,
    program::{GenFunction, StatementIdx, TypeDeclaration},
};
use rayon::iter::IntoParallelRefIterator;
use rayon::iter::ParallelIterator;
use std::{collections::HashMap, sync::Arc};
use universal_sierra_compiler_api::compile_raw_sierra_at_path;

#[tracing::instrument(skip_all, level = "debug")]
pub fn test_target_with_config(
    test_target_raw: TestTargetRaw,
    tracked_resource: &ForgeTrackedResource,
) -> Result<TestTargetWithConfig> {
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

    let casm_program = Arc::new(compile_raw_sierra_at_path(
        test_target_raw.sierra_program_path.as_std_path(),
    )?);

    let default_executables = vec![];
    let executables = test_target_raw
        .sierra_program
        .debug_info
        .as_ref()
        .and_then(|info| info.executables.get("snforge_internal_test_executable"))
        .unwrap_or(&default_executables);

    let test_cases = executables
        .par_iter()
        .map(|case| -> Result<TestCaseWithConfig> {
            let func = funcs[&case.id];

            let test_details = build_test_details(func, &type_declarations);

            let raw_config = run_config_pass(&test_details, &casm_program, tracked_resource)?;

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
        sierra_program_path: test_target_raw.sierra_program_path.into(),
        casm_program,
    })
}

#[tracing::instrument(skip_all, level = "debug")]
fn build_test_details(
    func: &GenFunction<StatementIdx>,
    type_declarations: &HashMap<u64, &TypeDeclaration>,
) -> TestDetails {
    let map_types = |concrete_types: &[ConcreteTypeId]| {
        concrete_types
            .iter()
            .map(|ty| {
                let ty = type_declarations[&ty.id];
                ty.long_id.generic_id.clone()
            })
            .collect()
    };

    TestDetails {
        sierra_entry_point_statement_idx: func.entry_point.0,
        parameter_types: map_types(&func.signature.param_types),
    }
}
