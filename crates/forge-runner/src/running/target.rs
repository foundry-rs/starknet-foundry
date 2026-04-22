use crate::{
    forge_config::ForgeTrackedResource,
    package_tests::{
        TestDetails,
        raw::TestTargetRaw,
        with_config::{TestCaseWithConfig, TestTargetWithConfig},
        with_config_resolved::sanitize_test_case_name,
    },
    running::config_run::run_config_pass,
};
use anyhow::Result;
use cairo_lang_sierra::{
    ids::ConcreteTypeId,
    program::{GenFunction, StatementIdx, TypeDeclaration},
};
use rayon::iter::IntoParallelIterator;
use rayon::iter::IntoParallelRefIterator;
use rayon::iter::ParallelIterator;
use std::{collections::HashMap, sync::Arc};
use universal_sierra_compiler_api::compile_raw_sierra_at_path;
use universal_sierra_compiler_api::representation::{AssembledCairoProgram, RawCasmProgram};

#[derive(Debug, Clone, Copy)]
pub enum TestSelectionMode<'a> {
    All,
    ExactMatch(&'a str),
}

#[tracing::instrument(skip_all, level = "debug")]
pub fn prepare_test_target(
    test_target_raw: TestTargetRaw,
    tracked_resource: &ForgeTrackedResource,
    test_selection_mode: TestSelectionMode<'_>,
) -> Result<TestTargetWithConfig> {
    let default_executables = vec![];
    let executables = test_target_raw
        .sierra_program
        .debug_info
        .as_ref()
        .and_then(|info| info.executables.get("snforge_internal_test_executable"))
        .unwrap_or(&default_executables);

    let selected_test_cases = match test_selection_mode {
        TestSelectionMode::All => None,
        TestSelectionMode::ExactMatch(exact_match) => Some(
            executables
                .iter()
                .filter(|case| {
                    let name: String = case.debug_name.clone().unwrap().into();
                    sanitize_test_case_name(&name) == exact_match
                })
                .collect::<Vec<_>>(),
        ),
    };

    if selected_test_cases.as_ref().is_some_and(Vec::is_empty) {
        return Ok(empty_test_target(test_target_raw));
    }

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

    let test_cases = if let Some(exact_matches) = selected_test_cases {
        exact_matches
            .into_par_iter()
            .map(|case| {
                build_test_case_with_config(
                    funcs[&case.id],
                    case.debug_name.clone().unwrap().into(),
                    &type_declarations,
                    &casm_program,
                    *tracked_resource,
                )
            })
            .collect::<Result<_>>()?
    } else {
        executables
            .par_iter()
            .map(|case| {
                build_test_case_with_config(
                    funcs[&case.id],
                    case.debug_name.clone().unwrap().into(),
                    &type_declarations,
                    &casm_program,
                    *tracked_resource,
                )
            })
            .collect::<Result<_>>()?
    };

    Ok(TestTargetWithConfig {
        tests_location: test_target_raw.tests_location,
        test_cases,
        sierra_program: test_target_raw.sierra_program,
        sierra_program_path: test_target_raw.sierra_program_path.into(),
        casm_program,
    })
}

fn empty_test_target(test_target_raw: TestTargetRaw) -> TestTargetWithConfig {
    // For non-matching `--exact` targets, return an empty test target.
    TestTargetWithConfig {
        tests_location: test_target_raw.tests_location,
        test_cases: vec![],
        sierra_program: test_target_raw.sierra_program,
        sierra_program_path: test_target_raw.sierra_program_path.into(),
        casm_program: Arc::new(RawCasmProgram {
            assembled_cairo_program: AssembledCairoProgram {
                bytecode: vec![],
                hints: vec![],
            },
            debug_info: vec![],
        }),
    }
}

fn build_test_case_with_config(
    func: &GenFunction<StatementIdx>,
    name: String,
    type_declarations: &HashMap<u64, &TypeDeclaration>,
    casm_program: &Arc<RawCasmProgram>,
    tracked_resource: ForgeTrackedResource,
) -> Result<TestCaseWithConfig> {
    let test_details = build_test_details(func, type_declarations);
    let raw_config = run_config_pass(&test_details, casm_program, &tracked_resource)?;

    Ok(TestCaseWithConfig {
        config: raw_config.into(),
        name,
        test_details,
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
