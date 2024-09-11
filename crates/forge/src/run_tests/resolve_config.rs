use crate::{block_number_map::BlockNumberMap, scarb::config::ForkTarget};
use anyhow::{anyhow, Result};
use cheatnet::runtime_extensions::forge_config_extension::config::{
    BlockId, InlineForkConfig, RawForkConfig,
};
use forge_runner::package_tests::{
    with_config::TestTargetWithConfig,
    with_config_resolved::{
        ResolvedForkConfig, TestCaseResolvedConfig, TestCaseWithResolvedConfig,
        TestTargetWithResolvedConfig,
    },
};
use num_bigint::BigInt;
use starknet_api::block::BlockNumber;

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
                    case.config.fork_config,
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
        casm_program: test_target.casm_program,
        test_cases,
    })
}

async fn resolve_fork_config(
    fork_config: Option<RawForkConfig>,
    block_number_map: &mut BlockNumberMap,
    fork_targets: &[ForkTarget],
) -> Result<Option<ResolvedForkConfig>> {
    let Some(fc) = fork_config else {
        return Ok(None);
    };

    let raw_fork_params = replace_id_with_params(fc, fork_targets)?;

    let url = raw_fork_params.url;

    let block_number = match raw_fork_params.block {
        BlockId::BlockNumber(block_number) => BlockNumber(block_number),
        BlockId::BlockHash(hash) => {
            block_number_map
                .get_block_number_for_hash(url.clone(), hash)
                .await?
        }
        BlockId::BlockTag => {
            block_number_map
                .get_latest_block_number(url.clone())
                .await?
        }
    };

    Ok(Some(ResolvedForkConfig { url, block_number }))
}

fn parse_block_id(fork_target: &ForkTarget) -> Result<BlockId> {
    let block_id = match fork_target.block_id_type.as_str() {
        "number" => BlockId::BlockNumber(fork_target.block_id_value.parse()?),
        "hash" => {
            let block_hash = fork_target.block_id_value.parse::<BigInt>()?;

            BlockId::BlockHash(block_hash.into())
        }
        "tag" => {
            if fork_target.block_id_value == "latest" {
                BlockId::BlockTag
            } else {
                Err(anyhow!(r#"only "latest" block tag is supported"#))?
            }
        }
        _ => Err(anyhow!("block_id must be one of (number | hash | tag)"))?,
    };

    Ok(block_id)
}

fn replace_id_with_params(
    raw_fork_config: RawForkConfig,
    fork_targets: &[ForkTarget],
) -> Result<InlineForkConfig> {
    match raw_fork_config {
        RawForkConfig::Inline(raw_fork_params) => Ok(raw_fork_params),
        RawForkConfig::Named(name) => {
            let fork_target_from_runner_config = fork_targets
                .iter()
                .find(|fork| fork.name == String::from(name.clone()))
                .ok_or_else(|| {
                    let name = String::from(name);

                    anyhow!("Fork configuration named = {name} not found in the Scarb.toml")
                })?;

            let block_id = parse_block_id(fork_target_from_runner_config)?;

            Ok(InlineForkConfig {
                url: fork_target_from_runner_config.url.parse()?,
                block: block_id,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cairo_lang_sierra::program::ProgramArtifact;
    use cairo_lang_sierra::{ids::GenericTypeId, program::Program};
    use forge_runner::package_tests::with_config::{TestCaseConfig, TestCaseWithConfig};
    use forge_runner::package_tests::TestTargetLocation;
    use forge_runner::{expected_result::ExpectedTestResult, package_tests::TestDetails};
    use std::sync::Arc;
    use universal_sierra_compiler_api::compile_sierra_to_casm;

    fn program_for_testing() -> ProgramArtifact {
        ProgramArtifact {
            program: Program {
                type_declarations: vec![],
                libfunc_declarations: vec![],
                statements: vec![],
                funcs: vec![],
            },
            debug_info: None,
        }
    }

    #[tokio::test]
    async fn to_runnable_unparsable_url() {
        let mocked_tests = TestTargetWithConfig {
            sierra_program: program_for_testing(),
            casm_program: Arc::new(compile_sierra_to_casm(&program_for_testing().program).unwrap()),
            test_cases: vec![TestCaseWithConfig {
                name: "crate1::do_thing".to_string(),
                config: TestCaseConfig {
                    available_gas: None,
                    ignored: false,
                    expected_result: ExpectedTestResult::Success,
                    fork_config: Some(RawForkConfig::Named("SOME_NAME".into())),
                    fuzzer_config: None,
                },
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
            mocked_tests,
            &[ForkTarget {
                name: "SOME_NAME".to_string(),
                url: "unparsable_url".to_string(),
                block_id_type: "Tag".to_string(),
                block_id_value: "Latest".to_string(),
            }],
            &mut BlockNumberMap::default()
        )
        .await
        .is_err());
    }

    #[tokio::test]
    async fn to_runnable_non_existent_id() {
        let mocked_tests = TestTargetWithConfig {
            sierra_program: program_for_testing(),
            casm_program: Arc::new(compile_sierra_to_casm(&program_for_testing().program).unwrap()),
            test_cases: vec![TestCaseWithConfig {
                name: "crate1::do_thing".to_string(),
                config: TestCaseConfig {
                    available_gas: None,
                    ignored: false,
                    expected_result: ExpectedTestResult::Success,
                    fork_config: Some(RawForkConfig::Named("non_existent".into())),
                    fuzzer_config: None,
                },
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
            mocked_tests,
            &[ForkTarget::new(
                "definitely_non_existing".to_string(),
                "https://not_taken.com".to_string(),
                "Number".to_string(),
                "120".to_string(),
            )],
            &mut BlockNumberMap::default()
        )
        .await
        .is_err());
    }
}
