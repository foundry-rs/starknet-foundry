use crate::{block_number_map::BlockNumberMap, scarb::config::ForkTarget};
use anyhow::{anyhow, Result};
use forge_runner::package_tests::{
    raw::{RawForkConfig, RawForkParams},
    with_config::TestTargetWithConfig,
    with_config_resolved::{
        ResolvedForkConfig, TestCaseResolvedConfig, TestCaseWithResolvedConfig,
        TestTargetWithResolvedConfig,
    },
};

pub async fn resolve_config(
    compiled_test_crate: TestTargetWithConfig,
    fork_targets: &[ForkTarget],
    block_number_map: &mut BlockNumberMap,
) -> Result<TestTargetWithResolvedConfig> {
    let mut test_cases = Vec::with_capacity(compiled_test_crate.test_cases.len());

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
) -> Result<Option<ResolvedForkConfig>> {
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

fn replace_id_with_params<'a>(
    raw_fork_config: &'a RawForkConfig,
    fork_targets: &'a [ForkTarget],
) -> Result<&'a RawForkParams> {
    match raw_fork_config {
        RawForkConfig::Params(raw_fork_params) => Ok(raw_fork_params),
        RawForkConfig::Id(name) => {
            let fork_target_from_runner_config = fork_targets
                .iter()
                .find(|fork| fork.name() == name)
                .ok_or_else(|| {
                    anyhow!("Fork configuration named = {name} not found in the Scarb.toml")
                })?;

            Ok(fork_target_from_runner_config.params())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cairo_lang_sierra::program::{ProgramArtifact, Version, VersionedProgram};
    use cairo_lang_sierra::{ids::GenericTypeId, program::Program};
    use forge_runner::package_tests::raw::{RawForkParams, TestCaseRaw, TestCrateRaw};
    use forge_runner::package_tests::TestTargetLocation;
    use forge_runner::{expected_result::ExpectedTestResult, package_tests::TestDetails};

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
        let mocked_tests = TestCrateRaw {
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
            tests_location: TestTargetLocation::Lib,
        };

        assert!(resolve_config(
            mocked_tests.with_config(),
            &[],
            &mut BlockNumberMap::default()
        )
        .await
        .is_err());
    }

    #[tokio::test]
    async fn to_runnable_non_existent_id() {
        let mocked_tests = TestCrateRaw {
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
            tests_location: TestTargetLocation::Lib,
        };

        assert!(resolve_config(
            mocked_tests.with_config(),
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
