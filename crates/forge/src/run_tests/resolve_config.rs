use std::sync::Arc;

use crate::run_tests::maat::env_ignore_fork_tests;
use crate::{
    block_number_map::BlockNumberMap, scarb::config::ForkTarget, test_filter::TestsFilter,
};
use anyhow::{Result, anyhow};
use cheatnet::runtime_extensions::forge_config_extension::config::{
    BlockId, InlineForkConfig, OverriddenForkConfig, RawForgeConfig, RawForkConfig,
};
use conversions::byte_array::ByteArray;
use forge_runner::TestCaseFilter;
use forge_runner::forge_config::ForgeTrackedResource;
use forge_runner::package_tests::with_config_resolved::{
    ResolvedForkConfig, TestCaseResolvedConfig,
};
use forge_runner::package_tests::{TestCandidate, TestCase, TestTarget};
use forge_runner::running::config_run::run_config_pass;
use starknet_api::block::BlockNumber;
use universal_sierra_compiler_api::compile_raw_sierra_at_path;

#[tracing::instrument(skip_all, level = "debug")]
pub async fn resolve_config(
    test_target: TestTarget<TestCandidate>,
    fork_targets: &[ForkTarget],
    block_number_map: &mut BlockNumberMap,
    tests_filter: &TestsFilter,
    tracked_resource: &ForgeTrackedResource,
) -> Result<TestTarget<TestCase>> {
    let mut test_cases = Vec::with_capacity(test_target.test_cases.len());
    let env_ignore_fork_tests = env_ignore_fork_tests();

    let casm_program = Arc::new(compile_raw_sierra_at_path(
        test_target.sierra_program_path.as_std_path(),
    )?);

    for test_candidate in test_target.test_cases {
        let raw_config = run_config_pass(
            &test_candidate.test_details,
            &casm_program.clone(),
            tracked_resource,
        )?;
        let resolved_config = resolved_config_from_raw(
            raw_config,
            tests_filter,
            fork_targets,
            block_number_map,
            env_ignore_fork_tests,
        )
        .await?;

        let test_case = TestCase::new(
            &test_candidate.name,
            test_candidate.test_details,
            resolved_config,
        );

        test_cases.push(test_case);
    }

    Ok(TestTarget {
        tests_location: test_target.tests_location,
        sierra_program: test_target.sierra_program,
        sierra_program_path: test_target.sierra_program_path,
        casm_program: Some(casm_program),
        test_cases,
    })
}

pub async fn resolved_config_from_raw(
    raw_config: RawForgeConfig,
    tests_filter: &TestsFilter,
    fork_targets: &[ForkTarget],
    block_number_map: &mut BlockNumberMap,
    env_ignore_fork_tests: bool,
) -> Result<TestCaseResolvedConfig> {
    let ignored = raw_config.ignore.is_some_and(|v| v.is_ignored)
        || (env_ignore_fork_tests && raw_config.fork.is_some());
    let fork_config = if tests_filter.should_run(ignored) {
        resolve_fork_config(raw_config.fork, block_number_map, fork_targets).await?
    } else {
        None
    };

    let resolved_config = TestCaseResolvedConfig {
        available_gas: raw_config.available_gas,
        ignored,
        fork_config,
        expected_result: raw_config.should_panic.into(),
        fuzzer_config: raw_config.fuzzer,
        disable_predeployed_contracts: raw_config
            .disable_predeployed_contracts
            .is_some_and(|v| v.is_disabled),
    };

    Ok(resolved_config)
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
    use cheatnet::runtime_extensions::forge_config_extension::config::RawIgnoreConfig;
    use url::Url;

    fn create_raw_config(ignored: bool, fork_config: Option<RawForkConfig>) -> RawForgeConfig {
        RawForgeConfig {
            available_gas: None,
            ignore: Some(RawIgnoreConfig {
                is_ignored: ignored,
            }),
            fork: fork_config,
            fuzzer: None,
            disable_predeployed_contracts: None,
            should_panic: None,
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
        let raw_config =
            create_raw_config(false, Some(RawForkConfig::Named("non_existent".into())));

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
            resolved_config_from_raw(
                raw_config,
                &tests_filter,
                &[create_fork_target(
                    "definitely_non_existing",
                    "https://not_taken.com",
                    BlockId::BlockNumber(120)
                )],
                &mut BlockNumberMap::default(),
                false
            )
            .await
            .is_err()
        );
    }

    #[tokio::test]
    async fn test_ignored_filter_skips_fork_config_resolution() {
        let ignored_raw_config =
            create_raw_config(true, Some(RawForkConfig::Named("non_existent_fork".into())));

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

        let resolved_config = resolved_config_from_raw(
            ignored_raw_config,
            &tests_filter,
            &[],
            &mut BlockNumberMap::default(),
            false,
        )
        .await
        .unwrap();

        assert!(resolved_config.ignored);
        assert!(resolved_config.fork_config.is_none());
    }

    #[tokio::test]
    async fn test_non_ignored_filter_resolves_fork_config() {
        let raw_config = create_raw_config(false, Some(RawForkConfig::Named("valid_fork".into())));

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

        let resolved_config = resolved_config_from_raw(
            raw_config,
            &tests_filter,
            &fork_targets,
            &mut BlockNumberMap::default(),
            false,
        )
        .await
        .unwrap();

        assert!(!resolved_config.ignored);
        assert!(resolved_config.fork_config.is_some());

        let fork_config = resolved_config.fork_config.as_ref().unwrap();
        assert_eq!(fork_config.url.as_str(), "https://example.com/");
        assert_eq!(fork_config.block_number.0, 100);
    }

    #[tokio::test]
    async fn test_name_filtered_test_still_resolves_fork_config() {
        let raw_config = create_raw_config(false, Some(RawForkConfig::Named("valid_fork".into())));
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

        let resolved_config = resolved_config_from_raw(
            raw_config,
            &tests_filter,
            &fork_targets,
            &mut BlockNumberMap::default(),
            false,
        )
        .await
        .unwrap();

        assert!(!resolved_config.ignored);
        assert!(resolved_config.fork_config.is_some());

        let fork_config = resolved_config.fork_config.as_ref().unwrap();
        assert_eq!(fork_config.url.as_str(), "https://example.com/");
        assert_eq!(fork_config.block_number.0, 100);
    }

    #[tokio::test]
    async fn test_mixed_scenarios_with_ignored_filter() {
        let raw_configs = vec![
            create_raw_config(true, Some(RawForkConfig::Named("valid_fork".into()))),
            create_raw_config(false, Some(RawForkConfig::Named("valid_fork".into()))),
            create_raw_config(false, Some(RawForkConfig::Named("valid_fork".into()))),
            create_raw_config(false, None),
        ];

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

        let mut resolved_configs = Vec::with_capacity(raw_configs.len());
        for raw_config in raw_configs {
            let resolved = resolved_config_from_raw(
                raw_config,
                &tests_filter,
                &fork_targets,
                &mut BlockNumberMap::default(),
                false,
            )
            .await
            .unwrap();

            resolved_configs.push(resolved);
        }

        // Check ignored test - should have no fork config resolved
        assert!(&resolved_configs[0].ignored);
        assert!(&resolved_configs[0].fork_config.is_none());

        // Check matching test - should have fork config resolved
        assert!(!&resolved_configs[1].ignored);
        assert!(&resolved_configs[1].fork_config.is_some());

        // Check non-matching test - should still have fork config resolved (name filtering happens later)
        assert!(!&resolved_configs[2].ignored);
        assert!(&resolved_configs[2].fork_config.is_some());

        // Check no-fork test - should work fine
        assert!(!&resolved_configs[3].ignored);
        assert!(&resolved_configs[3].fork_config.is_none());
    }

    #[tokio::test]
    async fn test_only_ignored_filter_skips_non_ignored_fork_resolution() {
        let raw_configs = vec![
            create_raw_config(true, Some(RawForkConfig::Named("valid_fork".into()))),
            create_raw_config(false, Some(RawForkConfig::Named("valid_fork".into()))),
        ];

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

        let mut resolved_configs = Vec::with_capacity(raw_configs.len());

        for raw_config in raw_configs {
            let resolved = resolved_config_from_raw(
                raw_config,
                &tests_filter,
                &fork_targets,
                &mut BlockNumberMap::default(),
                false,
            )
            .await
            .unwrap();

            resolved_configs.push(resolved);
        }

        // Ignored test should have fork config resolved since it should be run
        assert!(&resolved_configs[0].ignored);
        assert!(&resolved_configs[0].fork_config.is_some());

        // Non-ignored test should not have fork config resolved since it won't be run
        assert!(!&resolved_configs[1].ignored);
        assert!(&resolved_configs[1].fork_config.is_none());
    }

    #[tokio::test]
    async fn test_include_ignored_filter_resolves_all_fork_configs() {
        let raw_configs = vec![
            create_raw_config(true, Some(RawForkConfig::Named("valid_fork".into()))),
            create_raw_config(false, Some(RawForkConfig::Named("valid_fork".into()))),
        ];

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

        let mut resolved_configs = Vec::with_capacity(raw_configs.len());
        for raw_config in raw_configs {
            let resolved = resolved_config_from_raw(
                raw_config,
                &tests_filter,
                &fork_targets,
                &mut BlockNumberMap::default(),
                false,
            )
            .await
            .unwrap();

            resolved_configs.push(resolved);
        }

        for resolved_config in &resolved_configs {
            assert!(resolved_config.fork_config.is_some());
            let fork_config = resolved_config.fork_config.as_ref().unwrap();
            assert_eq!(fork_config.url.as_str(), "https://example.com/");
            assert_eq!(fork_config.block_number.0, 500);
        }

        assert!(&resolved_configs[0].ignored);
        assert!(!&resolved_configs[1].ignored);
    }

    #[tokio::test]
    async fn test_fork_config_resolution_with_inline_config() {
        let raw_config = create_raw_config(
            false,
            Some(RawForkConfig::Inline(InlineForkConfig {
                url: "https://inline-fork.com".parse().unwrap(),
                block: BlockId::BlockNumber(123),
            })),
        );

        let tests_filter = TestsFilter::from_flags(
            None,
            false,
            Vec::new(),
            false,
            false,
            false,
            FailedTestsCache::default(),
        );

        let resolved_config = resolved_config_from_raw(
            raw_config,
            &tests_filter,
            &[],
            &mut BlockNumberMap::default(),
            false,
        )
        .await
        .unwrap();

        assert!(!resolved_config.ignored);
        assert!(resolved_config.fork_config.is_some());

        let fork_config = resolved_config.fork_config.as_ref().unwrap();
        assert_eq!(fork_config.url.as_str(), "https://inline-fork.com/");
        assert_eq!(fork_config.block_number.0, 123);
    }

    #[tokio::test]
    async fn test_overridden_fork_config_resolution() {
        let raw_config = create_raw_config(
            false,
            Some(RawForkConfig::Overridden(OverriddenForkConfig {
                name: "base_fork".into(),
                block: BlockId::BlockNumber(999),
            })),
        );

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

        let resolved_config = resolved_config_from_raw(
            raw_config,
            &tests_filter,
            &fork_targets,
            &mut BlockNumberMap::default(),
            false,
        )
        .await
        .unwrap();

        assert!(resolved_config.fork_config.is_some());

        let fork_config = resolved_config.fork_config.as_ref().unwrap();
        assert_eq!(fork_config.url.as_str(), "https://base-fork.com/");
        assert_eq!(fork_config.block_number.0, 999);
    }

    #[tokio::test]
    async fn test_skip_filter_does_not_affect_fork_resolution() {
        let raw_config = create_raw_config(false, Some(RawForkConfig::Named("valid_fork".into())));

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

        let resolved_config = resolved_config_from_raw(
            raw_config,
            &tests_filter,
            &fork_targets,
            &mut BlockNumberMap::default(),
            false,
        )
        .await
        .unwrap();

        assert!(!resolved_config.ignored);
        assert!(resolved_config.fork_config.is_some());

        let fork_config = resolved_config.fork_config.as_ref().unwrap();
        assert_eq!(fork_config.url.as_str(), "https://example.com/");
        assert_eq!(fork_config.block_number.0, 600);
    }
}
