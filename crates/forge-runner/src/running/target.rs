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
    ids::{ConcreteTypeId, FunctionId},
    program::{GenFunction, StatementIdx, TypeDeclaration},
};
use rayon::iter::IntoParallelIterator;
use rayon::iter::ParallelIterator;
use std::{collections::HashMap, sync::Arc};
use universal_sierra_compiler_api::compile_raw_sierra_at_path;
use universal_sierra_compiler_api::representation::RawCasmProgram;

pub struct PrepareTestTargetResult {
    pub target: Option<TestTargetWithConfig>,
    pub location: TestTargetLocation,
    pub prefiltered_out_count: usize,
}

struct MatchedTestCase {
    id: u64,
    name: String,
}

#[tracing::instrument(skip_all, level = "debug")]
pub fn prepare_test_target(
    test_target_raw: TestTargetRaw,
    tracked_resource: &ForgeTrackedResource,
    name_filter: &NameFilter,
    partition_config: &PartitionConfig,
) -> Result<PrepareTestTargetResult> {
    let tests_location = test_target_raw.tests_location;
    let default_executables = vec![];
    let executables = test_target_raw
        .sierra_program
        .debug_info
        .as_ref()
        .and_then(|info| info.executables.get("snforge_internal_test_executable"))
        .unwrap_or(&default_executables);

    let (matched_cases, prefiltered_out_count) =
        collect_matched_cases(executables, name_filter, partition_config);

    if matched_cases.is_empty() {
        return Ok(PrepareTestTargetResult {
            target: None,
            location: tests_location,
            prefiltered_out_count,
        });
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

    let test_cases = matched_cases
        .into_par_iter()
        .map(|matched_test| {
            build_test_case_with_config(
                funcs[&matched_test.id],
                matched_test.name,
                &type_declarations,
                &casm_program,
                *tracked_resource,
            )
        })
        .collect::<Result<_>>()?;

    Ok(PrepareTestTargetResult {
        target: Some(TestTargetWithConfig {
            tests_location,
            test_cases,
            sierra_program: test_target_raw.sierra_program,
            sierra_program_path: test_target_raw.sierra_program_path.into(),
            casm_program,
        }),
        location: tests_location,
        prefiltered_out_count,
    })
}

fn collect_matched_cases(
    executables: &[FunctionId],
    name_filter: &NameFilter,
    partition_config: &PartitionConfig,
) -> (Vec<MatchedTestCase>, usize) {
    let is_in_partition = |test_name: &str| match partition_config {
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

    match name_filter {
        NameFilter::ExactMatch(_) => {
            let mut matched_case = None;
            let mut filtered_out_count = 0;

            for case in executables {
                let raw_name: String = case
                    .debug_name
                    .clone()
                    .expect("Failed to get test case name")
                    .into();
                let sanitized_name = sanitize_test_case_name(&raw_name);

                if name_filter.matches(&sanitized_name) {
                    matched_case = Some(MatchedTestCase {
                        id: case.id,
                        name: raw_name,
                    });
                } else {
                    filtered_out_count += 1;
                }
            }

            (matched_case.into_iter().collect(), filtered_out_count)
        }
        NameFilter::Match(_) => {
            let mut matched_cases = vec![];
            let mut filtered_out_count = 0;

            for case in executables {
                let debug_name: String = case
                    .debug_name
                    .clone()
                    .expect("Failed to get test case name")
                    .into();
                let sanitized_name = sanitize_test_case_name(&debug_name);

                if name_filter.matches(&sanitized_name) {
                    matched_cases.push(MatchedTestCase {
                        id: case.id,
                        name: debug_name,
                    });
                } else if is_in_partition(&sanitized_name) {
                    filtered_out_count += 1;
                }
            }

            (matched_cases, filtered_out_count)
        }
        NameFilter::All => (
            executables
                .iter()
                .map(|case| MatchedTestCase {
                    id: case.id,
                    name: case
                        .debug_name
                        .clone()
                        .expect("Failed to get test case name")
                        .into(),
                })
                .collect(),
            0,
        ),
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
