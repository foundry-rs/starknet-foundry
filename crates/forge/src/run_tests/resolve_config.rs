use std::sync::Arc;

use super::maat::env_ignore_fork_tests;
use crate::{
    block_number_map::BlockNumberMap, scarb::config::ForkTarget, test_filter::TestsFilter,
};
use anyhow::{Result, anyhow};
use cheatnet::runtime_extensions::forge_config_extension::config::{
    BlockId, InlineForkConfig, OverriddenForkConfig, RawForkConfig,
};
use conversions::byte_array::ByteArray;
use forge_runner::{
    TestCaseFilter,
    forge_config::ForgeTrackedResource,
    package_tests::{
        TestTargetWithTests,
        with_config_resolved::{
            ResolvedForkConfig, TestCaseResolvedConfig, TestCaseWithResolvedConfig,
            TestTargetWithResolvedConfig,
        },
    },
    running::config_run::run_config_pass,
};
use starknet_api::block::BlockNumber;
use universal_sierra_compiler_api::compile_raw_sierra_at_path;

#[tracing::instrument(skip_all, level = "debug")]
pub async fn resolve_config(
    test_target: TestTargetWithTests,
    fork_targets: &[ForkTarget],
    block_number_map: &mut BlockNumberMap,
    tests_filter: &TestsFilter,
    tracked_resource: &ForgeTrackedResource,
) -> Result<TestTargetWithResolvedConfig> {
    let mut test_cases = Vec::with_capacity(test_target.test_cases.len());
    let env_ignore_fork_tests = env_ignore_fork_tests();

    let casm_program = Arc::new(compile_raw_sierra_at_path(
        test_target.sierra_program_path.as_std_path(),
    )?);

    for case in test_target.test_cases {
        let raw_config = run_config_pass(&case.test_details, &casm_program, tracked_resource)?;
        let ignored = raw_config.ignore.is_some_and(|v| v.is_ignored);

        test_cases.push(TestCaseWithResolvedConfig::new(
            &case.name,
            case.test_details.clone(),
            TestCaseResolvedConfig {
                available_gas: raw_config.available_gas,
                ignored: ignored || (env_ignore_fork_tests && raw_config.fork.is_some()),
                fork_config: if tests_filter.should_run_test(ignored) {
                    resolve_fork_config(raw_config.fork, block_number_map, fork_targets).await?
                } else {
                    None
                },
                expected_result: raw_config.should_panic.into(),
                fuzzer_config: raw_config.fuzzer,
                disable_predeployed_contracts: raw_config
                    .disable_predeployed_contracts
                    .is_some_and(|v| v.is_disabled),
            },
        ));
    }

    Ok(TestTargetWithResolvedConfig {
        tests_location: test_target.tests_location,
        sierra_program: test_target.sierra_program,
        sierra_program_path: test_target.sierra_program_path,
        casm_program: casm_program,
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
    use crate::shared_cache::FailedTestsCache;
    use cairo_lang_sierra::program::ProgramArtifact;
    use cairo_lang_sierra::{ids::GenericTypeId, program::Program};
    use forge_runner::package_tests::TestTargetLocation;
    use forge_runner::package_tests::with_config::{
        TestCaseConfig, TestCaseWithConfig, TestTargetWithConfig,
    };
    use forge_runner::{expected_result::ExpectedTestResult, package_tests::TestDetails};
    use std::sync::Arc;
    use universal_sierra_compiler_api::compile_raw_sierra;
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

    fn create_test_case_with_config(
        name: &str,
        ignored: bool,
        fork_config: Option<RawForkConfig>,
    ) -> TestCaseWithConfig {
        TestCaseWithConfig {
            name: name.to_string(),
            config: TestCaseConfig {
                available_gas: None,
                ignored,
                expected_result: ExpectedTestResult::Success,
                fork_config,
                fuzzer_config: None,
                disable_predeployed_contracts: false,
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
        }
    }

    fn create_test_target_with_cases(test_cases: Vec<TestCaseWithConfig>) -> TestTargetWithConfig {
        TestTargetWithConfig {
            sierra_program: program_for_testing(),
            sierra_program_path: Arc::default(),
            casm_program: Arc::new(
                compile_raw_sierra(&serde_json::to_value(&program_for_testing().program).unwrap())
                    .unwrap(),
            ),
            test_cases,
            tests_location: TestTargetLocation::Lib,
        }
    }

    fn create_fork_target(name: &str, url: &str, block_id: BlockId) -> ForkTarget {
        ForkTarget {
            name: name.to_string(),
            url: Url::parse(url).expect("Should be valid url"),
            block_id,
        }
    }

    #[tokio::test]
    async fn to_runnable_non_existent_id() {
        let mocked_tests = create_test_target_with_cases(vec![create_test_case_with_config(
            "crate1::do_thing",
            false,
            Some(RawForkConfig::Named("non_existent".into())),
        )]);

        let tests_filter = TestsFilter::from_flags(
            None,
            false,
            Vec::new(),
            false,
            false,
            false,
            FailedTestsCache::default(),
        );

        assert!(
            resolve_config(
                mocked_tests,
                &[create_fork_target(
                    "definitely_non_existing",
                    "https://not_taken.com",
                    BlockId::BlockNumber(120)
                )],
                &mut BlockNumberMap::default(),
                &tests_filter,
            )
            .await
            .is_err()
        );
    }

    #[tokio::test]
    async fn test_ignored_filter_skips_fork_config_resolution() {
        let ignored_test = create_test_case_with_config(
            "ignored_test",
            true,
            Some(RawForkConfig::Named("non_existent_fork".into())),
        );

        let test_target = create_test_target_with_cases(vec![ignored_test]);

        // Create a filter that excludes ignored tests
        let tests_filter = TestsFilter::from_flags(
            None,
            false,
            Vec::new(),
            false,
            false,
            false,
            FailedTestsCache::default(),
        );

        let resolved = resolve_config(
            test_target,
            &[],
            &mut BlockNumberMap::default(),
            &tests_filter,
        )
        .await
        .unwrap();

        assert_eq!(resolved.test_cases.len(), 1);
        assert!(resolved.test_cases[0].config.ignored);
        assert!(resolved.test_cases[0].config.fork_config.is_none());
    }

    #[tokio::test]
    async fn test_non_ignored_filter_resolves_fork_config() {
        let test_case = create_test_case_with_config(
            "valid_test",
            false,
            Some(RawForkConfig::Named("valid_fork".into())),
        );

        let test_target = create_test_target_with_cases(vec![test_case]);

        let fork_targets = vec![create_fork_target(
            "valid_fork",
            "https://example.com",
            BlockId::BlockNumber(100),
        )];

        let tests_filter = TestsFilter::from_flags(
            None,
            false,
            Vec::new(),
            false,
            true,
            false,
            FailedTestsCache::default(),
        );

        let resolved = resolve_config(
            test_target,
            &fork_targets,
            &mut BlockNumberMap::default(),
            &tests_filter,
        )
        .await
        .unwrap();

        assert_eq!(resolved.test_cases.len(), 1);
        assert!(!resolved.test_cases[0].config.ignored);
        assert!(resolved.test_cases[0].config.fork_config.is_some());

        let fork_config = resolved.test_cases[0].config.fork_config.as_ref().unwrap();
        assert_eq!(fork_config.url.as_str(), "https://example.com/");
        assert_eq!(fork_config.block_number.0, 100);
    }

    #[tokio::test]
    async fn test_name_filtered_test_still_resolves_fork_config() {
        let test_case = create_test_case_with_config(
            "filtered_out_test",
            false,
            Some(RawForkConfig::Named("valid_fork".into())),
        );

        let test_target = create_test_target_with_cases(vec![test_case]);

        let fork_targets = vec![create_fork_target(
            "valid_fork",
            "https://example.com",
            BlockId::BlockNumber(100),
        )];

        let tests_filter = TestsFilter::from_flags(
            Some("different_pattern".to_string()),
            false,
            Vec::new(),
            false,
            false,
            false,
            FailedTestsCache::default(),
        );

        let resolved = resolve_config(
            test_target,
            &fork_targets,
            &mut BlockNumberMap::default(),
            &tests_filter,
        )
        .await
        .unwrap();

        assert_eq!(resolved.test_cases.len(), 1);
        assert!(!resolved.test_cases[0].config.ignored);
        assert!(resolved.test_cases[0].config.fork_config.is_some());

        let fork_config = resolved.test_cases[0].config.fork_config.as_ref().unwrap();
        assert_eq!(fork_config.url.as_str(), "https://example.com/");
        assert_eq!(fork_config.block_number.0, 100);
    }

    #[tokio::test]
    async fn test_mixed_scenarios_with_ignored_filter() {
        let test_cases = vec![
            create_test_case_with_config(
                "ignored_with_valid_fork",
                true,
                Some(RawForkConfig::Named("valid_fork".into())),
            ),
            create_test_case_with_config(
                "matching_test",
                false,
                Some(RawForkConfig::Named("valid_fork".into())),
            ),
            create_test_case_with_config(
                "non_matching_test",
                false,
                Some(RawForkConfig::Named("valid_fork".into())),
            ),
            create_test_case_with_config("no_fork_test", false, None),
        ];

        let test_target = create_test_target_with_cases(test_cases);
        let fork_targets = vec![create_fork_target(
            "valid_fork",
            "https://example.com",
            BlockId::BlockNumber(200),
        )];

        let tests_filter = TestsFilter::from_flags(
            Some("matching".to_string()),
            false,
            Vec::new(),
            false,
            false,
            false,
            FailedTestsCache::default(),
        );

        let resolved = resolve_config(
            test_target,
            &fork_targets,
            &mut BlockNumberMap::default(),
            &tests_filter,
        )
        .await
        .unwrap();

        assert_eq!(resolved.test_cases.len(), 4);

        // Check ignored test - should have no fork config resolved
        let ignored_test = &resolved.test_cases[0];
        assert_eq!(ignored_test.name, "ignored_with_valid_fork");
        assert!(ignored_test.config.ignored);
        assert!(ignored_test.config.fork_config.is_none());

        // Check matching test - should have fork config resolved
        let matching_test = &resolved.test_cases[1];
        assert_eq!(matching_test.name, "matching_test");
        assert!(!matching_test.config.ignored);
        assert!(matching_test.config.fork_config.is_some());

        // Check non-matching test - should still have fork config resolved (name filtering happens later)
        let non_matching_test = &resolved.test_cases[2];
        assert_eq!(non_matching_test.name, "non_matching_test");
        assert!(!non_matching_test.config.ignored);
        assert!(non_matching_test.config.fork_config.is_some());

        // Check no-fork test - should work fine
        let no_fork_test = &resolved.test_cases[3];
        assert_eq!(no_fork_test.name, "no_fork_test");
        assert!(!no_fork_test.config.ignored);
        assert!(no_fork_test.config.fork_config.is_none());
    }

    #[tokio::test]
    async fn test_only_ignored_filter_skips_non_ignored_fork_resolution() {
        let test_cases = vec![
            create_test_case_with_config(
                "ignored_test",
                true,
                Some(RawForkConfig::Named("valid_fork".into())),
            ),
            create_test_case_with_config(
                "non_ignored_test",
                false,
                Some(RawForkConfig::Named("valid_fork".into())),
            ),
        ];

        let test_target = create_test_target_with_cases(test_cases);
        let fork_targets = vec![create_fork_target(
            "valid_fork",
            "https://example.com",
            BlockId::BlockNumber(400),
        )];

        let tests_filter = TestsFilter::from_flags(
            None,
            false,
            Vec::new(),
            true,
            false,
            false,
            FailedTestsCache::default(),
        );

        let resolved = resolve_config(
            test_target,
            &fork_targets,
            &mut BlockNumberMap::default(),
            &tests_filter,
        )
        .await
        .unwrap();

        assert_eq!(resolved.test_cases.len(), 2);

        // Ignored test should have fork config resolved since it should be run
        let ignored_test = &resolved.test_cases[0];
        assert_eq!(ignored_test.name, "ignored_test");
        assert!(ignored_test.config.ignored);
        assert!(ignored_test.config.fork_config.is_some());

        // Non-ignored test should not have fork config resolved since it won't be run
        let non_ignored_test = &resolved.test_cases[1];
        assert_eq!(non_ignored_test.name, "non_ignored_test");
        assert!(!non_ignored_test.config.ignored);
        assert!(non_ignored_test.config.fork_config.is_none());
    }

    #[tokio::test]
    async fn test_include_ignored_filter_resolves_all_fork_configs() {
        let test_cases = vec![
            create_test_case_with_config(
                "ignored_test",
                true,
                Some(RawForkConfig::Named("valid_fork".into())),
            ),
            create_test_case_with_config(
                "non_ignored_test",
                false,
                Some(RawForkConfig::Named("valid_fork".into())),
            ),
        ];

        let test_target = create_test_target_with_cases(test_cases);

        let fork_targets = vec![create_fork_target(
            "valid_fork",
            "https://example.com",
            BlockId::BlockNumber(500),
        )];

        let tests_filter = TestsFilter::from_flags(
            None,
            false,
            Vec::new(),
            false,
            true,
            false,
            FailedTestsCache::default(),
        );

        let resolved = resolve_config(
            test_target,
            &fork_targets,
            &mut BlockNumberMap::default(),
            &tests_filter,
        )
        .await
        .unwrap();

        assert_eq!(resolved.test_cases.len(), 2);

        for test_case in &resolved.test_cases {
            assert!(test_case.config.fork_config.is_some());
            let fork_config = test_case.config.fork_config.as_ref().unwrap();
            assert_eq!(fork_config.url.as_str(), "https://example.com/");
            assert_eq!(fork_config.block_number.0, 500);
        }

        let ignored_test = &resolved.test_cases[0];
        assert_eq!(ignored_test.name, "ignored_test");
        assert!(ignored_test.config.ignored);

        let non_ignored_test = &resolved.test_cases[1];
        assert_eq!(non_ignored_test.name, "non_ignored_test");
        assert!(!non_ignored_test.config.ignored);
    }

    #[tokio::test]
    async fn test_fork_config_resolution_with_inline_config() {
        let test_case = create_test_case_with_config(
            "test_with_inline_fork",
            false,
            Some(RawForkConfig::Inline(InlineForkConfig {
                url: "https://inline-fork.com".parse().unwrap(),
                block: BlockId::BlockNumber(123),
            })),
        );

        let test_target = create_test_target_with_cases(vec![test_case]);
        let tests_filter = TestsFilter::from_flags(
            None,
            false,
            Vec::new(),
            false,
            false,
            false,
            FailedTestsCache::default(),
        );

        let resolved = resolve_config(
            test_target,
            &[],
            &mut BlockNumberMap::default(),
            &tests_filter,
        )
        .await
        .unwrap();

        assert_eq!(resolved.test_cases.len(), 1);

        let test_case = &resolved.test_cases[0];
        assert!(!test_case.config.ignored);
        assert!(test_case.config.fork_config.is_some());

        let fork_config = test_case.config.fork_config.as_ref().unwrap();
        assert_eq!(fork_config.url.as_str(), "https://inline-fork.com/");
        assert_eq!(fork_config.block_number.0, 123);
    }

    #[tokio::test]
    async fn test_overridden_fork_config_resolution() {
        let test_case = create_test_case_with_config(
            "test_with_overridden_fork",
            false,
            Some(RawForkConfig::Overridden(OverriddenForkConfig {
                name: "base_fork".into(),
                block: BlockId::BlockNumber(999),
            })),
        );

        let test_target = create_test_target_with_cases(vec![test_case]);
        let fork_targets = vec![create_fork_target(
            "base_fork",
            "https://base-fork.com",
            BlockId::BlockNumber(100),
        )];

        let tests_filter = TestsFilter::from_flags(
            None,
            false,
            Vec::new(),
            false,
            false,
            false,
            FailedTestsCache::default(),
        );

        let resolved = resolve_config(
            test_target,
            &fork_targets,
            &mut BlockNumberMap::default(),
            &tests_filter,
        )
        .await
        .unwrap();

        assert_eq!(resolved.test_cases.len(), 1);
        let test_case = &resolved.test_cases[0];
        assert!(test_case.config.fork_config.is_some());

        let fork_config = test_case.config.fork_config.as_ref().unwrap();
        assert_eq!(fork_config.url.as_str(), "https://base-fork.com/");
        assert_eq!(fork_config.block_number.0, 999);
    }

    #[tokio::test]
    async fn test_skip_filter_does_not_affect_fork_resolution() {
        let test_case = create_test_case_with_config(
            "test_to_be_skipped",
            false,
            Some(RawForkConfig::Named("valid_fork".into())),
        );

        let test_target = create_test_target_with_cases(vec![test_case]);

        let fork_targets = vec![create_fork_target(
            "valid_fork",
            "https://example.com",
            BlockId::BlockNumber(600),
        )];

        let tests_filter = TestsFilter::from_flags(
            None,
            false,
            vec!["skipped".to_string()],
            false,
            false,
            false,
            FailedTestsCache::default(),
        );

        let resolved = resolve_config(
            test_target,
            &fork_targets,
            &mut BlockNumberMap::default(),
            &tests_filter,
        )
        .await
        .unwrap();

        assert_eq!(resolved.test_cases.len(), 1);

        let test_case = &resolved.test_cases[0];
        assert!(!test_case.config.ignored);
        assert!(test_case.config.fork_config.is_some());

        let fork_config = test_case.config.fork_config.as_ref().unwrap();
        assert_eq!(fork_config.url.as_str(), "https://example.com/");
        assert_eq!(fork_config.block_number.0, 600);
    }
}
