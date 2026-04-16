use crate::shared_cache::FailedTestsCache;
use anyhow::Result;
use forge_runner::filtering::{ExcludeReason, FilterResult, TestCaseFilter, TestCaseIsIgnored};
use forge_runner::package_tests::TestCase;
use forge_runner::package_tests::with_config_resolved::{
    TestCaseWithResolvedConfig, sanitize_test_case_name,
};
use forge_runner::partition::PartitionConfig;

#[derive(Debug, PartialEq, Clone)]
// Specifies what tests should be included
pub struct TestsFilter {
    // based on name
    pub(crate) name_filter: NameFilter,
    // based on `#[ignore]` attribute
    ignored_filter: IgnoredFilter,
    // based on `--rerun_failed` flag
    last_failed_filter: bool,
    // based on `--skip` flag
    skip_filter: Vec<String>,

    failed_tests_cache: FailedTestsCache,
    pub(crate) partitioning_config: PartitionConfig,
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) enum NameFilter {
    All,
    Match(String),
    ExactMatch(String),
}

#[derive(Debug, PartialEq, Clone)]
pub enum IgnoredFilter {
    IncludeAll,
    IgnoredOnly,
    ExcludeIgnored,
}

pub(crate) struct PreConfigTestFilter {
    name_filter: NameFilter,
    skip_filter: Vec<String>,
    failed_tests: Option<Vec<String>>,
    partition_config: PartitionConfig,
}

impl TestsFilter {
    #[must_use]
    #[expect(clippy::fn_params_excessive_bools)]
    #[expect(clippy::too_many_arguments)]
    pub fn from_flags(
        test_name_filter: Option<String>,
        exact_match: bool,
        skip: Vec<String>,
        only_ignored: bool,
        include_ignored: bool,
        rerun_failed: bool,
        failed_tests_cache: FailedTestsCache,
        partitioning_config: PartitionConfig,
    ) -> Self {
        assert!(
            !(only_ignored && include_ignored),
            "Arguments only_ignored and include_ignored cannot be both true"
        );

        let ignored_filter = if include_ignored {
            IgnoredFilter::IncludeAll
        } else if only_ignored {
            IgnoredFilter::IgnoredOnly
        } else {
            IgnoredFilter::ExcludeIgnored
        };

        let name_filter = if exact_match {
            NameFilter::ExactMatch(
                test_name_filter
                    .expect("Argument test_name_filter cannot be None with exact_match"),
            )
        } else if let Some(name) = test_name_filter {
            NameFilter::Match(name)
        } else {
            NameFilter::All
        };

        Self {
            name_filter,
            ignored_filter,
            last_failed_filter: rerun_failed,
            skip_filter: skip,
            failed_tests_cache,
            partitioning_config,
        }
    }

    pub(crate) fn pre_config_filter(&self) -> Result<PreConfigTestFilter> {
        let failed_tests = if self.last_failed_filter {
            Some(self.failed_tests_cache.load()?)
        } else {
            None
        };

        Ok(PreConfigTestFilter {
            name_filter: self.name_filter.clone(),
            skip_filter: self.skip_filter.clone(),
            failed_tests,
            partition_config: self.partitioning_config.clone(),
        })
    }

    pub(crate) fn has_name_filter(&self) -> bool {
        !matches!(&self.name_filter, NameFilter::All)
    }

    pub(crate) fn filter_tests(
        &self,
        test_cases: &mut Vec<TestCaseWithResolvedConfig>,
    ) -> Result<()> {
        match &self.name_filter {
            NameFilter::All => {}
            NameFilter::Match(filter) => {
                test_cases.retain(|tc| tc.name.contains(filter));
            }

            NameFilter::ExactMatch(name) => {
                test_cases.retain(|tc| tc.name == *name);
            }
        }

        if self.last_failed_filter {
            match self.failed_tests_cache.load()?.as_slice() {
                [] => {}
                result => {
                    test_cases.retain(|tc| result.iter().any(|name| name == &tc.name));
                }
            }
        }

        match self.ignored_filter {
            // if ExcludeIgnored (default) we filter ignored tests later and display them as ignored
            IgnoredFilter::IncludeAll | IgnoredFilter::ExcludeIgnored => {}
            IgnoredFilter::IgnoredOnly => {
                test_cases.retain(|tc| tc.config.ignored);
            }
        }

        if !self.skip_filter.is_empty() {
            test_cases.retain(|tc| !self.skip_filter.iter().any(|s| tc.name.contains(s)));
        }

        Ok(())
    }
}

impl PreConfigTestFilter {
    pub(crate) fn includes(&self, raw_name: &str) -> bool {
        let name = sanitize_test_case_name(raw_name);

        if !self.matches_name_filter(&name) {
            return false;
        }

        if self.skip_filter.iter().any(|skip| name.contains(skip)) {
            return false;
        }

        if !self.matches_failed_tests_filter(&name) {
            return false;
        }

        self.partition_config.includes_test(&name)
    }

    fn matches_name_filter(&self, name: &str) -> bool {
        match &self.name_filter {
            NameFilter::All => true,
            NameFilter::Match(filter) => name.contains(filter),
            NameFilter::ExactMatch(exact) => name == exact,
        }
    }

    fn matches_failed_tests_filter(&self, name: &str) -> bool {
        match &self.failed_tests {
            Some(failed_tests) if !failed_tests.is_empty() => {
                failed_tests.iter().any(|failed_test| failed_test == name)
            }
            Some(_) | None => true,
        }
    }
}

impl TestCaseFilter for TestsFilter {
    fn filter<T>(&self, test_case: &TestCase<T>) -> FilterResult
    where
        T: TestCaseIsIgnored,
    {
        // Order of filter checks matters, because we do not want to display a test as ignored if
        // it was excluded due to partitioning.
        if !self.partitioning_config.includes_test(&test_case.name) {
            return FilterResult::Excluded(ExcludeReason::ExcludedFromPartition);
        }

        let case_ignored = test_case.config.is_ignored();

        match self.ignored_filter {
            IgnoredFilter::IncludeAll => {}
            IgnoredFilter::IgnoredOnly => {
                if !case_ignored {
                    return FilterResult::Excluded(ExcludeReason::Ignored);
                }
            }
            IgnoredFilter::ExcludeIgnored => {
                if case_ignored {
                    return FilterResult::Excluded(ExcludeReason::Ignored);
                }
            }
        }
        FilterResult::Included
    }
}

#[cfg(test)]
mod tests {
    use crate::shared_cache::FailedTestsCache;
    use crate::test_filter::TestsFilter;
    use cairo_lang_sierra::program::{Program, ProgramArtifact};
    use camino::Utf8PathBuf;
    use forge_runner::expected_result::ExpectedTestResult;
    use forge_runner::package_tests::with_config_resolved::{
        TestCaseResolvedConfig, TestCaseWithResolvedConfig, TestTargetWithResolvedConfig,
    };
    use forge_runner::package_tests::{TestDetails, TestTargetLocation};
    use forge_runner::partition::PartitionConfig;
    use std::sync::Arc;
    use universal_sierra_compiler_api::compile_raw_sierra;

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

    #[test]
    #[should_panic(expected = "Arguments only_ignored and include_ignored cannot be both true")]
    fn from_flags_only_ignored_and_include_ignored_both_true() {
        let _ = TestsFilter::from_flags(
            None,
            false,
            Vec::new(),
            true,
            true,
            false,
            FailedTestsCache::default(),
            PartitionConfig::default(),
        );
    }

    #[test]
    #[should_panic(expected = "Argument test_name_filter cannot be None with exact_match")]
    fn from_flags_exact_match_true_without_test_filter_name() {
        let _ = TestsFilter::from_flags(
            None,
            true,
            Vec::new(),
            false,
            false,
            false,
            FailedTestsCache::default(),
            PartitionConfig::default(),
        );
    }

    #[test]
    fn pre_config_filter_applies_name_skip_and_failed_tests_filters() {
        let temp_dir = tempfile::tempdir().unwrap();
        let cache_dir = Utf8PathBuf::from_path_buf(temp_dir.path().to_path_buf()).unwrap();
        std::fs::write(cache_dir.join(".prev_tests_failed"), "crate::matched\n").unwrap();

        let tests_filter = TestsFilter::from_flags(
            Some("matched".to_string()),
            false,
            vec!["skip".to_string()],
            false,
            false,
            true,
            FailedTestsCache::new(&cache_dir),
            PartitionConfig::default(),
        );

        let pre_config_filter = tests_filter.pre_config_filter().unwrap();

        assert!(pre_config_filter.includes("crate::matched__snforge_internal_test_generated"));
        assert!(!pre_config_filter.includes("crate::not_matched"));
        assert!(!pre_config_filter.includes("crate::matched_skip"));
        assert!(!pre_config_filter.includes("crate::matched_but_not_failed"));
    }

    #[test]
    #[expect(clippy::too_many_lines)]
    fn filtering_tests() {
        let mocked_tests = TestTargetWithResolvedConfig {
            sierra_program: program_for_testing(),
            sierra_program_path: Arc::default(),
            casm_program: Arc::new(
                compile_raw_sierra(&serde_json::to_value(&program_for_testing().program).unwrap())
                    .unwrap(),
            ),
            test_cases: vec![
                TestCaseWithResolvedConfig {
                    name: "crate1::do_thing".to_string(),
                    test_details: TestDetails::default(),

                    config: TestCaseResolvedConfig {
                        available_gas: None,
                        ignored: false,
                        expected_result: ExpectedTestResult::Success,
                        fork_config: None,
                        fuzzer_config: None,
                        disable_predeployed_contracts: false,
                    },
                },
                TestCaseWithResolvedConfig {
                    name: "crate2::run_other_thing".to_string(),
                    test_details: TestDetails::default(),

                    config: TestCaseResolvedConfig {
                        available_gas: None,
                        ignored: true,
                        expected_result: ExpectedTestResult::Success,
                        fork_config: None,
                        fuzzer_config: None,
                        disable_predeployed_contracts: false,
                    },
                },
                TestCaseWithResolvedConfig {
                    name: "outer::crate2::execute_next_thing".to_string(),
                    test_details: TestDetails::default(),

                    config: TestCaseResolvedConfig {
                        available_gas: None,
                        ignored: true,
                        expected_result: ExpectedTestResult::Success,
                        fork_config: None,
                        fuzzer_config: None,
                        disable_predeployed_contracts: false,
                    },
                },
                TestCaseWithResolvedConfig {
                    name: "thing".to_string(),
                    test_details: TestDetails::default(),

                    config: TestCaseResolvedConfig {
                        available_gas: None,
                        ignored: false,
                        expected_result: ExpectedTestResult::Success,
                        fork_config: None,
                        fuzzer_config: None,
                        disable_predeployed_contracts: false,
                    },
                },
            ],
            tests_location: TestTargetLocation::Lib,
        };

        let tests_filter = TestsFilter::from_flags(
            Some("do".to_string()),
            false,
            Vec::new(),
            false,
            false,
            false,
            FailedTestsCache::default(),
            PartitionConfig::default(),
        );

        let mut filtered = mocked_tests.clone();

        tests_filter.filter_tests(&mut filtered.test_cases).unwrap();

        assert_eq!(
            filtered.test_cases,
            vec![TestCaseWithResolvedConfig {
                name: "crate1::do_thing".to_string(),
                test_details: TestDetails::default(),

                config: TestCaseResolvedConfig {
                    available_gas: None,
                    ignored: false,
                    expected_result: ExpectedTestResult::Success,
                    fork_config: None,
                    fuzzer_config: None,
                    disable_predeployed_contracts: false,
                },
            },]
        );

        let tests_filter = TestsFilter::from_flags(
            Some("te2::run".to_string()),
            false,
            Vec::new(),
            false,
            false,
            false,
            FailedTestsCache::default(),
            PartitionConfig::default(),
        );

        let mut filtered = mocked_tests.clone();
        tests_filter.filter_tests(&mut filtered.test_cases).unwrap();

        assert_eq!(
            filtered.test_cases,
            vec![TestCaseWithResolvedConfig {
                name: "crate2::run_other_thing".to_string(),
                test_details: TestDetails::default(),

                config: TestCaseResolvedConfig {
                    available_gas: None,
                    ignored: true,
                    expected_result: ExpectedTestResult::Success,
                    fork_config: None,
                    fuzzer_config: None,
                    disable_predeployed_contracts: false,
                },
            },]
        );

        let tests_filter = TestsFilter::from_flags(
            Some("thing".to_string()),
            false,
            Vec::new(),
            false,
            false,
            false,
            FailedTestsCache::default(),
            PartitionConfig::default(),
        );

        let mut filtered = mocked_tests.clone();
        tests_filter.filter_tests(&mut filtered.test_cases).unwrap();

        assert_eq!(
            filtered.test_cases,
            vec![
                TestCaseWithResolvedConfig {
                    name: "crate1::do_thing".to_string(),
                    test_details: TestDetails::default(),

                    config: TestCaseResolvedConfig {
                        available_gas: None,
                        ignored: false,
                        expected_result: ExpectedTestResult::Success,
                        fork_config: None,
                        fuzzer_config: None,
                        disable_predeployed_contracts: false,
                    },
                },
                TestCaseWithResolvedConfig {
                    name: "crate2::run_other_thing".to_string(),
                    test_details: TestDetails::default(),

                    config: TestCaseResolvedConfig {
                        available_gas: None,
                        ignored: true,
                        expected_result: ExpectedTestResult::Success,
                        fork_config: None,
                        fuzzer_config: None,
                        disable_predeployed_contracts: false,
                    },
                },
                TestCaseWithResolvedConfig {
                    name: "outer::crate2::execute_next_thing".to_string(),
                    test_details: TestDetails::default(),

                    config: TestCaseResolvedConfig {
                        available_gas: None,
                        ignored: true,
                        expected_result: ExpectedTestResult::Success,
                        fork_config: None,
                        fuzzer_config: None,
                        disable_predeployed_contracts: false,
                    },
                },
                TestCaseWithResolvedConfig {
                    name: "thing".to_string(),
                    test_details: TestDetails::default(),

                    config: TestCaseResolvedConfig {
                        available_gas: None,
                        ignored: false,
                        expected_result: ExpectedTestResult::Success,
                        fork_config: None,
                        fuzzer_config: None,
                        disable_predeployed_contracts: false,
                    },
                },
            ]
        );

        let tests_filter = TestsFilter::from_flags(
            Some("nonexistent".to_string()),
            false,
            Vec::new(),
            false,
            false,
            false,
            FailedTestsCache::default(),
            PartitionConfig::default(),
        );

        let mut filtered = mocked_tests.clone();
        tests_filter.filter_tests(&mut filtered.test_cases).unwrap();

        assert_eq!(filtered.test_cases, vec![]);

        let tests_filter = TestsFilter::from_flags(
            Some(String::new()),
            false,
            Vec::new(),
            false,
            false,
            false,
            FailedTestsCache::default(),
            PartitionConfig::default(),
        );

        let mut filtered = mocked_tests.clone();
        tests_filter.filter_tests(&mut filtered.test_cases).unwrap();

        assert_eq!(
            filtered.test_cases,
            vec![
                TestCaseWithResolvedConfig {
                    name: "crate1::do_thing".to_string(),
                    test_details: TestDetails::default(),

                    config: TestCaseResolvedConfig {
                        available_gas: None,
                        ignored: false,
                        expected_result: ExpectedTestResult::Success,
                        fork_config: None,
                        fuzzer_config: None,
                        disable_predeployed_contracts: false,
                    },
                },
                TestCaseWithResolvedConfig {
                    name: "crate2::run_other_thing".to_string(),
                    test_details: TestDetails::default(),

                    config: TestCaseResolvedConfig {
                        available_gas: None,
                        ignored: true,
                        expected_result: ExpectedTestResult::Success,
                        fork_config: None,
                        fuzzer_config: None,
                        disable_predeployed_contracts: false,
                    },
                },
                TestCaseWithResolvedConfig {
                    name: "outer::crate2::execute_next_thing".to_string(),
                    test_details: TestDetails::default(),

                    config: TestCaseResolvedConfig {
                        available_gas: None,
                        ignored: true,
                        expected_result: ExpectedTestResult::Success,
                        fork_config: None,
                        fuzzer_config: None,
                        disable_predeployed_contracts: false,
                    },
                },
                TestCaseWithResolvedConfig {
                    name: "thing".to_string(),
                    test_details: TestDetails::default(),

                    config: TestCaseResolvedConfig {
                        available_gas: None,
                        ignored: false,
                        expected_result: ExpectedTestResult::Success,
                        fork_config: None,
                        fuzzer_config: None,
                        disable_predeployed_contracts: false,
                    },
                },
            ]
        );
    }

    #[test]
    fn filtering_with_no_tests() {
        let mocked_tests = TestTargetWithResolvedConfig {
            sierra_program: program_for_testing(),
            sierra_program_path: Arc::default(),
            casm_program: Arc::new(
                compile_raw_sierra(&serde_json::to_value(&program_for_testing().program).unwrap())
                    .unwrap(),
            ),
            test_cases: vec![],
            tests_location: TestTargetLocation::Lib,
        };

        let tests_filter = TestsFilter::from_flags(
            Some(String::new()),
            false,
            Vec::new(),
            false,
            false,
            false,
            FailedTestsCache::default(),
            PartitionConfig::default(),
        );

        let mut filtered = mocked_tests.clone();
        tests_filter.filter_tests(&mut filtered.test_cases).unwrap();

        assert_eq!(filtered.test_cases, vec![]);

        let tests_filter = TestsFilter::from_flags(
            Some("thing".to_string()),
            false,
            Vec::new(),
            false,
            false,
            false,
            FailedTestsCache::default(),
            PartitionConfig::default(),
        );

        let mut filtered = mocked_tests.clone();
        tests_filter.filter_tests(&mut filtered.test_cases).unwrap();

        assert_eq!(filtered.test_cases, vec![]);
    }

    #[test]
    #[expect(clippy::too_many_lines)]
    fn filtering_with_exact_match() {
        let mocked_tests = TestTargetWithResolvedConfig {
            sierra_program: program_for_testing(),
            sierra_program_path: Arc::default(),
            casm_program: Arc::new(
                compile_raw_sierra(&serde_json::to_value(&program_for_testing().program).unwrap())
                    .unwrap(),
            ),
            test_cases: vec![
                TestCaseWithResolvedConfig {
                    name: "crate1::do_thing".to_string(),
                    test_details: TestDetails::default(),

                    config: TestCaseResolvedConfig {
                        available_gas: None,
                        ignored: false,
                        expected_result: ExpectedTestResult::Success,
                        fork_config: None,
                        fuzzer_config: None,
                        disable_predeployed_contracts: false,
                    },
                },
                TestCaseWithResolvedConfig {
                    name: "crate2::run_other_thing".to_string(),
                    test_details: TestDetails::default(),

                    config: TestCaseResolvedConfig {
                        available_gas: None,
                        ignored: true,
                        expected_result: ExpectedTestResult::Success,
                        fork_config: None,
                        fuzzer_config: None,
                        disable_predeployed_contracts: false,
                    },
                },
                TestCaseWithResolvedConfig {
                    name: "outer::crate3::run_other_thing".to_string(),
                    test_details: TestDetails::default(),

                    config: TestCaseResolvedConfig {
                        available_gas: None,
                        ignored: true,
                        expected_result: ExpectedTestResult::Success,
                        fork_config: None,
                        fuzzer_config: None,
                        disable_predeployed_contracts: false,
                    },
                },
                TestCaseWithResolvedConfig {
                    name: "do_thing".to_string(),
                    test_details: TestDetails::default(),

                    config: TestCaseResolvedConfig {
                        available_gas: None,
                        ignored: false,
                        expected_result: ExpectedTestResult::Success,
                        fork_config: None,
                        fuzzer_config: None,
                        disable_predeployed_contracts: false,
                    },
                },
            ],
            tests_location: TestTargetLocation::Tests,
        };

        let tests_filter = TestsFilter::from_flags(
            Some(String::new()),
            true,
            Vec::new(),
            false,
            false,
            false,
            FailedTestsCache::default(),
            PartitionConfig::default(),
        );

        let mut filtered = mocked_tests.clone();
        tests_filter.filter_tests(&mut filtered.test_cases).unwrap();

        assert_eq!(filtered.test_cases, vec![]);

        let tests_filter = TestsFilter::from_flags(
            Some("thing".to_string()),
            true,
            Vec::new(),
            false,
            false,
            false,
            FailedTestsCache::default(),
            PartitionConfig::default(),
        );

        let mut filtered = mocked_tests.clone();
        tests_filter.filter_tests(&mut filtered.test_cases).unwrap();

        assert_eq!(filtered.test_cases, vec![]);

        let tests_filter = TestsFilter::from_flags(
            Some("do_thing".to_string()),
            true,
            Vec::new(),
            false,
            false,
            false,
            FailedTestsCache::default(),
            PartitionConfig::default(),
        );

        let mut filtered = mocked_tests.clone();
        tests_filter.filter_tests(&mut filtered.test_cases).unwrap();

        assert_eq!(
            filtered.test_cases,
            vec![TestCaseWithResolvedConfig {
                name: "do_thing".to_string(),
                test_details: TestDetails::default(),

                config: TestCaseResolvedConfig {
                    available_gas: None,
                    ignored: false,
                    expected_result: ExpectedTestResult::Success,
                    fork_config: None,
                    fuzzer_config: None,
                    disable_predeployed_contracts: false,
                },
            },]
        );

        let tests_filter = TestsFilter::from_flags(
            Some("crate1::do_thing".to_string()),
            true,
            Vec::new(),
            false,
            false,
            false,
            FailedTestsCache::default(),
            PartitionConfig::default(),
        );

        let mut filtered = mocked_tests.clone();
        tests_filter.filter_tests(&mut filtered.test_cases).unwrap();

        assert_eq!(
            filtered.test_cases,
            vec![TestCaseWithResolvedConfig {
                name: "crate1::do_thing".to_string(),
                test_details: TestDetails::default(),

                config: TestCaseResolvedConfig {
                    available_gas: None,
                    ignored: false,
                    expected_result: ExpectedTestResult::Success,
                    fork_config: None,
                    fuzzer_config: None,
                    disable_predeployed_contracts: false,
                },
            },]
        );

        let tests_filter = TestsFilter::from_flags(
            Some("crate3::run_other_thing".to_string()),
            true,
            Vec::new(),
            false,
            false,
            false,
            FailedTestsCache::default(),
            PartitionConfig::default(),
        );

        let mut filtered = mocked_tests.clone();
        tests_filter.filter_tests(&mut filtered.test_cases).unwrap();

        assert_eq!(filtered.test_cases, vec![]);

        let tests_filter = TestsFilter::from_flags(
            Some("outer::crate3::run_other_thing".to_string()),
            true,
            Vec::new(),
            false,
            false,
            false,
            FailedTestsCache::default(),
            PartitionConfig::default(),
        );

        let mut filtered = mocked_tests.clone();
        tests_filter.filter_tests(&mut filtered.test_cases).unwrap();

        assert_eq!(
            filtered.test_cases,
            vec![TestCaseWithResolvedConfig {
                name: "outer::crate3::run_other_thing".to_string(),
                test_details: TestDetails::default(),

                config: TestCaseResolvedConfig {
                    available_gas: None,
                    ignored: true,
                    expected_result: ExpectedTestResult::Success,
                    fork_config: None,
                    fuzzer_config: None,
                    disable_predeployed_contracts: false,
                },
            },]
        );
    }

    #[test]
    fn filtering_with_only_ignored() {
        let mocked_tests = TestTargetWithResolvedConfig {
            sierra_program: program_for_testing(),
            sierra_program_path: Arc::default(),
            casm_program: Arc::new(
                compile_raw_sierra(&serde_json::to_value(&program_for_testing().program).unwrap())
                    .unwrap(),
            ),
            test_cases: vec![
                TestCaseWithResolvedConfig {
                    name: "crate1::do_thing".to_string(),
                    test_details: TestDetails::default(),

                    config: TestCaseResolvedConfig {
                        available_gas: None,
                        ignored: false,
                        expected_result: ExpectedTestResult::Success,
                        fork_config: None,
                        fuzzer_config: None,
                        disable_predeployed_contracts: false,
                    },
                },
                TestCaseWithResolvedConfig {
                    name: "crate2::run_other_thing".to_string(),
                    test_details: TestDetails::default(),

                    config: TestCaseResolvedConfig {
                        available_gas: None,
                        ignored: true,
                        expected_result: ExpectedTestResult::Success,
                        fork_config: None,
                        fuzzer_config: None,
                        disable_predeployed_contracts: false,
                    },
                },
                TestCaseWithResolvedConfig {
                    name: "outer::crate3::run_other_thing".to_string(),
                    test_details: TestDetails::default(),

                    config: TestCaseResolvedConfig {
                        available_gas: None,
                        ignored: true,
                        expected_result: ExpectedTestResult::Success,
                        fork_config: None,
                        fuzzer_config: None,
                        disable_predeployed_contracts: false,
                    },
                },
                TestCaseWithResolvedConfig {
                    name: "do_thing".to_string(),
                    test_details: TestDetails::default(),

                    config: TestCaseResolvedConfig {
                        available_gas: None,
                        ignored: false,
                        expected_result: ExpectedTestResult::Success,
                        fork_config: None,
                        fuzzer_config: None,
                        disable_predeployed_contracts: false,
                    },
                },
            ],
            tests_location: TestTargetLocation::Tests,
        };

        let tests_filter = TestsFilter::from_flags(
            None,
            false,
            Vec::new(),
            true,
            false,
            false,
            FailedTestsCache::default(),
            PartitionConfig::default(),
        );
        let mut filtered = mocked_tests;
        tests_filter.filter_tests(&mut filtered.test_cases).unwrap();

        assert_eq!(
            filtered.test_cases,
            vec![
                TestCaseWithResolvedConfig {
                    name: "crate2::run_other_thing".to_string(),
                    test_details: TestDetails::default(),

                    config: TestCaseResolvedConfig {
                        available_gas: None,
                        ignored: true,
                        expected_result: ExpectedTestResult::Success,
                        fork_config: None,
                        fuzzer_config: None,
                        disable_predeployed_contracts: false,
                    },
                },
                TestCaseWithResolvedConfig {
                    name: "outer::crate3::run_other_thing".to_string(),
                    test_details: TestDetails::default(),

                    config: TestCaseResolvedConfig {
                        available_gas: None,
                        ignored: true,
                        expected_result: ExpectedTestResult::Success,
                        fork_config: None,
                        fuzzer_config: None,
                        disable_predeployed_contracts: false,
                    },
                },
            ]
        );
    }

    #[test]
    #[expect(clippy::too_many_lines)]
    fn filtering_with_include_ignored() {
        let mocked_tests = TestTargetWithResolvedConfig {
            sierra_program: program_for_testing(),
            sierra_program_path: Arc::default(),
            casm_program: Arc::new(
                compile_raw_sierra(&serde_json::to_value(&program_for_testing().program).unwrap())
                    .unwrap(),
            ),
            test_cases: vec![
                TestCaseWithResolvedConfig {
                    name: "crate1::do_thing".to_string(),
                    test_details: TestDetails::default(),

                    config: TestCaseResolvedConfig {
                        available_gas: None,
                        ignored: false,
                        expected_result: ExpectedTestResult::Success,
                        fork_config: None,
                        fuzzer_config: None,
                        disable_predeployed_contracts: false,
                    },
                },
                TestCaseWithResolvedConfig {
                    name: "crate2::run_other_thing".to_string(),
                    test_details: TestDetails::default(),

                    config: TestCaseResolvedConfig {
                        available_gas: None,
                        ignored: true,
                        expected_result: ExpectedTestResult::Success,
                        fork_config: None,
                        fuzzer_config: None,
                        disable_predeployed_contracts: false,
                    },
                },
                TestCaseWithResolvedConfig {
                    name: "outer::crate3::run_other_thing".to_string(),
                    test_details: TestDetails::default(),

                    config: TestCaseResolvedConfig {
                        available_gas: None,
                        ignored: true,
                        expected_result: ExpectedTestResult::Success,
                        fork_config: None,
                        fuzzer_config: None,
                        disable_predeployed_contracts: false,
                    },
                },
                TestCaseWithResolvedConfig {
                    name: "do_thing".to_string(),
                    test_details: TestDetails::default(),

                    config: TestCaseResolvedConfig {
                        available_gas: None,
                        ignored: false,
                        expected_result: ExpectedTestResult::Success,
                        fork_config: None,
                        fuzzer_config: None,
                        disable_predeployed_contracts: false,
                    },
                },
            ],
            tests_location: TestTargetLocation::Tests,
        };

        let tests_filter = TestsFilter::from_flags(
            None,
            false,
            Vec::new(),
            false,
            true,
            false,
            FailedTestsCache::default(),
            PartitionConfig::default(),
        );
        let mut filtered = mocked_tests;
        tests_filter.filter_tests(&mut filtered.test_cases).unwrap();

        assert_eq!(
            filtered.test_cases,
            vec![
                TestCaseWithResolvedConfig {
                    name: "crate1::do_thing".to_string(),
                    test_details: TestDetails::default(),

                    config: TestCaseResolvedConfig {
                        available_gas: None,
                        ignored: false,
                        expected_result: ExpectedTestResult::Success,
                        fork_config: None,
                        fuzzer_config: None,
                        disable_predeployed_contracts: false,
                    },
                },
                TestCaseWithResolvedConfig {
                    name: "crate2::run_other_thing".to_string(),
                    test_details: TestDetails::default(),

                    config: TestCaseResolvedConfig {
                        available_gas: None,
                        ignored: true,
                        expected_result: ExpectedTestResult::Success,
                        fork_config: None,
                        fuzzer_config: None,
                        disable_predeployed_contracts: false,
                    },
                },
                TestCaseWithResolvedConfig {
                    name: "outer::crate3::run_other_thing".to_string(),
                    test_details: TestDetails::default(),

                    config: TestCaseResolvedConfig {
                        available_gas: None,
                        ignored: true,
                        expected_result: ExpectedTestResult::Success,
                        fork_config: None,
                        fuzzer_config: None,
                        disable_predeployed_contracts: false,
                    },
                },
                TestCaseWithResolvedConfig {
                    name: "do_thing".to_string(),
                    test_details: TestDetails::default(),

                    config: TestCaseResolvedConfig {
                        available_gas: None,
                        ignored: false,
                        expected_result: ExpectedTestResult::Success,
                        fork_config: None,
                        fuzzer_config: None,
                        disable_predeployed_contracts: false,
                    },
                },
            ]
        );
    }
}
