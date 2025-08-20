use crate::shared_cache::FailedTestsCache;
use anyhow::Result;
use forge_runner::TestCaseFilter;
use forge_runner::package_tests::with_config_resolved::TestCaseWithResolvedConfig;

#[derive(Debug, PartialEq)]
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
}

#[derive(Debug, PartialEq)]
pub(crate) enum NameFilter {
    All,
    Match(String),
    ExactMatch(String),
}

#[derive(Debug, PartialEq)]
pub(crate) enum IgnoredFilter {
    NotIgnored,
    Ignored,
    All,
}

impl TestsFilter {
    #[must_use]
    #[expect(clippy::fn_params_excessive_bools)]
    pub fn from_flags(
        test_name_filter: Option<String>,
        exact_match: bool,
        skip: Vec<String>,
        only_ignored: bool,
        include_ignored: bool,
        rerun_failed: bool,
        failed_tests_cache: FailedTestsCache,
    ) -> Self {
        assert!(
            !(only_ignored && include_ignored),
            "Arguments only_ignored and include_ignored cannot be both true"
        );

        let ignored_filter = if include_ignored {
            IgnoredFilter::All
        } else if only_ignored {
            IgnoredFilter::Ignored
        } else {
            IgnoredFilter::NotIgnored
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
        }
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
            // if NotIgnored (default) we filter ignored tests later and display them as ignored
            IgnoredFilter::All | IgnoredFilter::NotIgnored => {}
            IgnoredFilter::Ignored => {
                test_cases.retain(|tc| tc.config.ignored);
            }
        }

        if !self.skip_filter.is_empty() {
            test_cases.retain(|tc| !self.skip_filter.iter().any(|s| tc.name.contains(s)));
        }

        Ok(())
    }
}

impl TestCaseFilter for TestsFilter {
    fn should_be_run(&self, test_case: &TestCaseWithResolvedConfig) -> bool {
        let ignored = test_case.config.ignored;

        match self.ignored_filter {
            IgnoredFilter::All => true,
            IgnoredFilter::Ignored => ignored,
            IgnoredFilter::NotIgnored => !ignored,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::shared_cache::FailedTestsCache;
    use crate::test_filter::TestsFilter;
    use cairo_lang_sierra::program::Program;
    use cairo_lang_sierra::program::ProgramArtifact;
    use cairo_native::context::NativeContext;
    use cairo_native::executor::AotNativeExecutor;
    use forge_runner::expected_result::ExpectedTestResult;
    use forge_runner::package_tests::with_config_resolved::{
        TestCaseResolvedConfig, TestCaseWithResolvedConfig, TestTargetWithResolvedConfig,
    };
    use forge_runner::package_tests::{TestDetails, TestTargetLocation};
    use std::sync::Arc;
    use universal_sierra_compiler_api::{SierraType, compile_sierra};

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

    fn executor_for_testing() -> Arc<AotNativeExecutor> {
        let native_context = NativeContext::new();
        let native_module = native_context
            .compile(&program_for_testing().program, true, None, None)
            .unwrap();
        let native_executor =
            AotNativeExecutor::from_native_module(native_module, cairo_native::OptLevel::Default)
                .unwrap();
        Arc::new(native_executor)
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
        );
    }

    #[test]
    #[expect(clippy::too_many_lines)]
    fn filtering_tests() {
        let mocked_tests = TestTargetWithResolvedConfig {
            sierra_program: program_for_testing(),
            sierra_program_path: Arc::default(),
            casm_program: Arc::new(
                compile_sierra(
                    &serde_json::to_value(&program_for_testing().program).unwrap(),
                    &SierraType::Raw,
                )
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
            aot_executor: executor_for_testing(),
        };

        let tests_filter = TestsFilter::from_flags(
            Some("do".to_string()),
            false,
            Vec::new(),
            false,
            false,
            false,
            FailedTestsCache::default(),
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
                compile_sierra(
                    &serde_json::to_value(&program_for_testing().program).unwrap(),
                    &SierraType::Raw,
                )
                .unwrap(),
            ),
            test_cases: vec![],
            tests_location: TestTargetLocation::Lib,
            aot_executor: executor_for_testing(),
        };

        let tests_filter = TestsFilter::from_flags(
            Some(String::new()),
            false,
            Vec::new(),
            false,
            false,
            false,
            FailedTestsCache::default(),
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
                compile_sierra(
                    &serde_json::to_value(&program_for_testing().program).unwrap(),
                    &SierraType::Raw,
                )
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
            aot_executor: executor_for_testing(),
        };

        let tests_filter = TestsFilter::from_flags(
            Some(String::new()),
            true,
            Vec::new(),
            false,
            false,
            false,
            FailedTestsCache::default(),
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
    #[expect(clippy::too_many_lines)]
    fn filtering_with_only_ignored() {
        let mocked_tests = TestTargetWithResolvedConfig {
            sierra_program: program_for_testing(),
            sierra_program_path: Arc::default(),
            casm_program: Arc::new(
                compile_sierra(
                    &serde_json::to_value(&program_for_testing().program).unwrap(),
                    &SierraType::Raw,
                )
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
            aot_executor: executor_for_testing(),
        };

        let tests_filter = TestsFilter::from_flags(
            None,
            false,
            Vec::new(),
            true,
            false,
            false,
            FailedTestsCache::default(),
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
                compile_sierra(
                    &serde_json::to_value(&program_for_testing().program).unwrap(),
                    &SierraType::Raw,
                )
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
            aot_executor: executor_for_testing(),
        };

        let tests_filter = TestsFilter::from_flags(
            None,
            false,
            Vec::new(),
            false,
            true,
            false,
            FailedTestsCache::default(),
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
