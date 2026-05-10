use crate::{
    filtering::NameFilter,
    forge_config::ForgeTrackedResource,
    package_tests::{
        TestDetails, TestTargetLocation,
        raw::TestTargetRaw,
        with_config::{TestCaseWithConfig, TestTargetWithConfig},
        with_config_resolved::sanitize_test_case_name,
    },
    partition::PartitionConfig,
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
use universal_sierra_compiler_api::representation::RawCasmProgram;

#[tracing::instrument(skip_all, level = "debug")]
#[expect(clippy::too_many_lines)]
pub fn prepare_test_target(
    test_target_raw: TestTargetRaw,
    tracked_resource: &ForgeTrackedResource,
    name_filter: &NameFilter,
    partitioning_config: &PartitionConfig,
) -> Result<(Option<TestTargetWithConfig>, TestTargetLocation, usize)> {
    let tests_location = test_target_raw.tests_location;
    let default_executables = vec![];
    let executables = test_target_raw
        .sierra_program
        .debug_info
        .as_ref()
        .and_then(|info| info.executables.get("snforge_internal_test_executable"))
        .unwrap_or(&default_executables);

    let is_in_partition = |test_name: &str| match partitioning_config {
        PartitionConfig::Disabled => true,
        PartitionConfig::Enabled {
            partition,
            partition_map,
        } => {
            let test_assigned_index = partition_map
                .get_assigned_index(test_name)
                .expect("Partition map must contain all test cases");
            test_assigned_index == partition.index()
        }
    };

    let (matching_cases, prefiltered_out_count) = match name_filter {
        NameFilter::ExactMatch(exact_match) => {
            let matches = executables
                .iter()
                .filter_map(|case| {
                    let raw_name: String = case.debug_name.clone()?.into();
                    let sanitized_name = sanitize_test_case_name(&raw_name);
                    (sanitized_name == *exact_match).then_some((&case.id, raw_name))
                })
                .collect::<Vec<_>>();

            if matches.is_empty() {
                return Ok((None, tests_location, 0));
            }

            (Some(matches), 0)
        }
        NameFilter::Match(filter) => {
            let mut matches = vec![];
            let mut filtered_out_count = 0;

            for case in executables {
                let Some(debug_name) = case.debug_name.clone() else {
                    continue;
                };
                let raw_name: String = debug_name.into();
                let sanitized_name = sanitize_test_case_name(&raw_name);

                if sanitized_name.contains(filter) {
                    matches.push((&case.id, raw_name));
                } else if is_in_partition(&sanitized_name) {
                    filtered_out_count += 1;
                }
            }

            if matches.is_empty() {
                return Ok((None, tests_location, filtered_out_count));
            }

            (Some(matches), filtered_out_count)
        }
        NameFilter::All => (None, 0),
    };

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

    let test_cases = if let Some(matches) = matching_cases {
        matches
            .into_par_iter()
            .map(|(id, name)| {
                build_test_case_with_config(
                    funcs[id],
                    name,
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
                    case.debug_name
                        .clone()
                        .expect("Failed to get test case name")
                        .into(),
                    &type_declarations,
                    &casm_program,
                    *tracked_resource,
                )
            })
            .collect::<Result<_>>()?
    };

    Ok((
        Some(TestTargetWithConfig {
            tests_location,
            test_cases,
            sierra_program: test_target_raw.sierra_program,
            sierra_program_path: test_target_raw.sierra_program_path.into(),
            casm_program,
        }),
        tests_location,
        prefiltered_out_count,
    ))
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
