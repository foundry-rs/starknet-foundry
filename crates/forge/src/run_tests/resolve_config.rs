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
use num_bigint::BigInt;
use starknet_api::block::BlockNumber;
use url::Url;

pub async fn resolve_config(
    test_target: TestTargetWithConfig,
    fork_targets: &[ForkTarget],
    block_number_map: &mut BlockNumberMap,
) -> Result<TestTargetWithResolvedConfig> {
    let mut test_cases = Vec::with_capacity(test_target.test_cases.len());

    for case in test_target.test_cases {
        test_cases.push(TestCaseWithResolvedConfig {
            name: case.name,
            test_details: case.test_details,
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
        tests_location: test_target.tests_location,
        sierra_program: test_target.sierra_program,
        test_cases,
    })
}

async fn resolve_fork_config(
    fork_config: &Option<RawForkConfig>,
    block_number_map: &mut BlockNumberMap,
    fork_targets: &[ForkTarget],
) -> Result<Option<ResolvedForkConfig>> {
    let Some(fc) = fork_config else {
        return Ok(None);
    };

    let raw_fork_params = replace_id_with_params(fc, fork_targets)?;

    let url: Url = raw_fork_params.url.parse()?;

    let block_number = match raw_fork_params.block_id_type.to_lowercase().as_str() {
        "number" => BlockNumber(raw_fork_params.block_id_value.parse()?),
        "hash" => {
            let block_hash = raw_fork_params.block_id_value.parse::<BigInt>()?;

            block_number_map
                .get_block_number_for_hash(url.clone(), block_hash.into())
                .await?
        }
        "tag" => {
            assert_eq!(raw_fork_params.block_id_value, "Latest");

            block_number_map
                .get_latest_block_number(url.clone())
                .await?
        }
        _ => panic!(),
    };

    Ok(Some(ResolvedForkConfig { url, block_number }))
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
    use forge_runner::package_tests::raw::{RawForkParams, TestCaseRaw, TestTargetRaw};
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
        let mocked_tests = TestTargetRaw {
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
        let mocked_tests = TestTargetRaw {
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
