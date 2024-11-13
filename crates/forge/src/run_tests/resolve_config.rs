use crate::{block_number_map::BlockNumberMap, scarb::config::ForkTarget};
use anyhow::{anyhow, Result};
use cheatnet::runtime_extensions::forge_config_extension::config::{
    BlockId, InlineForkConfig, OverriddenForkConfig, RawForkConfig,
};
use conversions::byte_array::ByteArray;
use forge_runner::package_tests::{
    with_config::TestTargetWithConfig,
    with_config_resolved::{
        ResolvedForkConfig, TestCaseResolvedConfig, TestCaseWithResolvedConfig,
        TestTargetWithResolvedConfig,
    },
};
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

fn get_fork_target_from_runner_config<'a>(
    fork_targets: &'a [ForkTarget],
    name: &ByteArray,
) -> Result<&'a ForkTarget> {
    fork_targets
        .iter()
        .find(|fork| fork.name == name.to_string())
        .ok_or_else(|| {
            let name = name.to_string();
            anyhow!("Fork configuration named = {name} not found in the Scarb.toml")
        })
}

fn replace_id_with_params(
    raw_fork_config: RawForkConfig,
    fork_targets: &[ForkTarget],
) -> Result<InlineForkConfig> {
    match raw_fork_config {
        RawForkConfig::Inline(raw_fork_params) => Ok(raw_fork_params),
        RawForkConfig::Named(name) => {
            let fork_target_from_runner_config =
                get_fork_target_from_runner_config(fork_targets, &name)?;

            let block_id = fork_target_from_runner_config.block_id.clone();

            Ok(InlineForkConfig {
                url: fork_target_from_runner_config.url.clone(),
                block: block_id,
            })
        }
        RawForkConfig::Overridden(OverriddenForkConfig { name, block }) => {
            let fork_target_from_runner_config =
                get_fork_target_from_runner_config(fork_targets, &name)?;

            let url = fork_target_from_runner_config.url.clone();

            Ok(InlineForkConfig { url, block })
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
    use universal_sierra_compiler_api::compile_sierra_program;
    use url::Url;

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
    async fn to_runnable_non_existent_id() {
        let mocked_tests = TestTargetWithConfig {
            sierra_program: program_for_testing(),
            casm_program: Arc::new(
                compile_sierra_program(&program_for_testing().program)
                    .unwrap()
                    .into(),
            ),
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
            &[ForkTarget {
                name: "definitely_non_existing".to_string(),
                url: Url::parse("https://not_taken.com").expect("Should be valid url"),
                block_id: BlockId::BlockNumber(120),
            }],
            &mut BlockNumberMap::default()
        )
        .await
        .is_err());
    }
}
