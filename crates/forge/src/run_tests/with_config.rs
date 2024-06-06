use super::replace_id_with_params;
use crate::{block_number_map::BlockNumberMap, scarb::config::ForkTarget};
use anyhow::Result;
use forge_runner::compiled_runnable::{
    ForkConfig, RawForkConfig, TestCaseResolvedConfig, TestCaseWithResolvedConfig,
    TestTargetWithConfig, TestTargetWithResolvedConfig,
};

pub async fn with_config(
    compiled_test_crate: TestTargetWithConfig,
    fork_targets: &[ForkTarget],
    block_number_map: &mut BlockNumberMap,
) -> Result<TestTargetWithResolvedConfig> {
    let mut test_cases = vec![];

    for case in compiled_test_crate.test_cases {
        test_cases.push(TestCaseWithResolvedConfig {
            test_case: case.test_case,
            config: TestCaseResolvedConfig {
                available_gas: case.config.available_gas,
                ignored: case.config.ignored,
                expected_result: case.config.expected_result,
                fork_config: resolve_fork_config(
                    &case.config.fork_config,
                    block_number_map,
                    fork_targets,
                )
                .await?,
                fuzzer_config: case.config.fuzzer_config,
            },
        });
    }

    Ok(TestTargetWithResolvedConfig {
        tests_location: compiled_test_crate.tests_location,
        sierra_program: compiled_test_crate.sierra_program,
        test_cases,
    })
}

async fn resolve_fork_config(
    fork_config: &Option<RawForkConfig>,
    block_number_map: &mut BlockNumberMap,
    fork_targets: &[ForkTarget],
) -> Result<Option<ForkConfig>> {
    let result = if let Some(fc) = fork_config {
        let raw_fork_params = replace_id_with_params(fc, fork_targets)?;
        let fork_config = block_number_map
            .validated_fork_config_from_fork_params(raw_fork_params)
            .await?;
        Some(fork_config)
    } else {
        None
    };

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use cairo_lang_sierra::program::{ProgramArtifact, Version, VersionedProgram};
    use cairo_lang_sierra::{ids::GenericTypeId, program::Program};
    use forge_runner::compiled_runnable::{
        CompiledTestCrateRaw, CrateLocation, RawForkParams, TestCaseRaw,
    };
    use forge_runner::{compiled_runnable::TestDetails, expected_result::ExpectedTestResult};

    fn program_for_testing() -> VersionedProgram {
        VersionedProgram::V1 {
            version: Version::<1>,
            program: ProgramArtifact {
                program: Program {
                    type_declarations: vec![],
                    libfunc_declarations: vec![],
                    statements: vec![],
                    funcs: vec![],
                },
                debug_info: None,
            },
        }
    }

    #[tokio::test]
    async fn to_runnable_unparsable_url() {
        let mocked_tests = CompiledTestCrateRaw {
            sierra_program: program_for_testing(),
            test_cases: vec![TestCaseRaw {
                name: "crate1::do_thing".to_string(),
                available_gas: None,
                ignored: false,
                expected_result: ExpectedTestResult::Success,
                fork_config: Some(RawForkConfig::Params(RawForkParams {
                    url: "unparsable_url".to_string(),
                    block_id_type: "Tag".to_string(),
                    block_id_value: "Latest".to_string(),
                })),
                fuzzer_config: None,
                test_details: TestDetails {
                    sierra_entry_point_statement_idx: 100,
                    parameter_types: vec![
                        (GenericTypeId("RangeCheck".into()), 1),
                        (GenericTypeId("GasBuiltin".into()), 1),
                    ],
                    return_types: vec![
                        (GenericTypeId("RangeCheck".into()), 1),
                        (GenericTypeId("GasBuiltin".into()), 1),
                        (GenericTypeId("Enum".into()), 3),
                    ],
                },
            }],
            tests_location: CrateLocation::Lib,
        };

        assert!(
            with_config(mocked_tests.into(), &[], &mut BlockNumberMap::default())
                .await
                .is_err()
        );
    }

    #[tokio::test]
    async fn to_runnable_non_existent_id() {
        let mocked_tests = CompiledTestCrateRaw {
            sierra_program: program_for_testing(),
            test_cases: vec![TestCaseRaw {
                name: "crate1::do_thing".to_string(),
                available_gas: None,
                ignored: false,
                expected_result: ExpectedTestResult::Success,
                fork_config: Some(RawForkConfig::Id("non_existent".to_string())),
                fuzzer_config: None,
                test_details: TestDetails {
                    sierra_entry_point_statement_idx: 100,
                    parameter_types: vec![
                        (GenericTypeId("RangeCheck".into()), 1),
                        (GenericTypeId("GasBuiltin".into()), 1),
                    ],
                    return_types: vec![
                        (GenericTypeId("RangeCheck".into()), 1),
                        (GenericTypeId("GasBuiltin".into()), 1),
                        (GenericTypeId("Enum".into()), 3),
                    ],
                },
            }],
            tests_location: CrateLocation::Lib,
        };

        assert!(with_config(
            mocked_tests.into(),
            &[ForkTarget::new(
                "definitely_non_existing".to_string(),
                RawForkParams {
                    url: "https://not_taken.com".to_string(),
                    block_id_type: "Number".to_string(),
                    block_id_value: "120".to_string(),
                },
            )],
            &mut BlockNumberMap::default()
        )
        .await
        .is_err());
    }
}
