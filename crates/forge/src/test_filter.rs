use crate::shared_cache::FailedTestsCache;
use anyhow::Result;
use forge_runner::{
    package_tests::with_config_resolved::TestCaseWithResolvedConfig, TestCaseFilter,
};

#[derive(Debug, PartialEq)]
// Specifies what tests should be included
pub struct TestsFilter {
    // based on name
    pub(crate) name_filter: NameFilter,
    // based on `#[ignore]` attribute
    ignored_filter: IgnoredFilter,
    // based on rerun_failed flag
    last_failed_filter: bool,

    failed_tests_cache: FailedTestsCache,
    // based on exclude filter
    exclude_filter: Option<NameFilter>,
}

#[derive(Debug, PartialEq)]
pub enum NameFilter {
    All,
    Match(String),
    ExactMatch(String),
    Exclude(String),
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
        only_ignored: bool,
        include_ignored: bool,
        rerun_failed: bool,
        failed_tests_cache: FailedTestsCache,
        exclude_filter: Option<String>,
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

        let exclude_filter = exclude_filter.map(NameFilter::Exclude);

        Self {
            name_filter,
            ignored_filter,
            last_failed_filter: rerun_failed,
            failed_tests_cache,
            exclude_filter,
        }
    }

    pub(crate) fn filter_tests(
        &self,
        test_cases: &mut Vec<TestCaseWithResolvedConfig>,
    ) -> Result<()> {
        match &self.name_filter {
            NameFilter::All | NameFilter::Exclude(_) => {}
            NameFilter::Match(filter) => {
                test_cases.retain(|tc| tc.name.contains(filter));
            }

            NameFilter::ExactMatch(name) => {
                test_cases.retain(|tc| tc.name == *name);
            }
        };

        if self.last_failed_filter {
            match self.failed_tests_cache.load()?.as_slice() {
                [] => {}
                result => {
                    test_cases.retain(|tc| result.iter().any(|name| name == &tc.name));
                }
            }
        }

        if let Some(NameFilter::Exclude(filter)) = &self.exclude_filter {
            test_cases.retain(|tc| !tc.name.contains(filter));
        }

        match self.ignored_filter {
            // if NotIgnored (default) we filter ignored tests later and display them as ignored
            IgnoredFilter::All | IgnoredFilter::NotIgnored => {}
            IgnoredFilter::Ignored => {
                test_cases.retain(|tc| tc.config.ignored);
            }
        };

        Ok(())
    }

    pub(crate) fn is_excluded(&self, test_case: &TestCaseWithResolvedConfig) -> bool {
        if let Some(NameFilter::Exclude(filter)) = &self.exclude_filter {
            return test_case.name.contains(filter);
        }
        false
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
    use crate::test_filter::TestsFilter;
    use cairo_lang_sierra::program::Program;
    use cairo_lang_sierra::program::ProgramArtifact;
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

    #[test]
    #[should_panic(expected = "Arguments only_ignored and include_ignored cannot be both true")]
    fn from_flags_only_ignored_and_include_ignored_both_true() {
        let _ = TestsFilter::from_flags(None, false, true, true, false, Default::default(), None);
    }

    #[test]
    #[should_panic(expected = "Argument test_name_filter cannot be None with exact_match")]
    fn from_flags_exact_match_true_without_test_filter_name() {
        let _ = TestsFilter::from_flags(None, true, false, false, false, Default::default(), None);
    }

    #[test]
    #[expect(clippy::too_many_lines)]
    fn filtering_tests() {
        let mocked_tests = TestTargetWithResolvedConfig {
            sierra_program: program_for_testing(),
            sierra_program_path: Default::default(),
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
                    },
                },
            ],
            tests_location: TestTargetLocation::Lib,
        };

        let tests_filter = TestsFilter::from_flags(
            Some("do".to_string()),
            false,
            false,
            false,
            false,
            Default::default(),
            None,
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
                },
            },]
        );

        let tests_filter = TestsFilter::from_flags(
            Some("te2::run".to_string()),
            false,
            false,
            false,
            false,
            Default::default(),
            None,
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
                },
            },]
        );

        let tests_filter = TestsFilter::from_flags(
            Some("thing".to_string()),
            false,
            false,
            false,
            false,
            Default::default(),
            None,
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
                    },
                },
            ]
        );

        let tests_filter = TestsFilter::from_flags(
            Some("nonexistent".to_string()),
            false,
            false,
            false,
            false,
            Default::default(),
            None,
        );

        let mut filtered = mocked_tests.clone();
        tests_filter.filter_tests(&mut filtered.test_cases).unwrap();

        assert_eq!(filtered.test_cases, vec![]);

        let tests_filter = TestsFilter::from_flags(
            Some(String::new()),
            false,
            false,
            false,
            false,
            Default::default(),
            None,
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
                    },
                },
            ]
        );
    }

    #[test]
    fn filtering_with_no_tests() {
        let mocked_tests = TestTargetWithResolvedConfig {
            sierra_program: program_for_testing(),
            sierra_program_path: Default::default(),
            casm_program: Arc::new(
                compile_sierra(
                    &serde_json::to_value(&program_for_testing().program).unwrap(),
                    &SierraType::Raw,
                )
                .unwrap(),
            ),
            test_cases: vec![],
            tests_location: TestTargetLocation::Lib,
        };

        let tests_filter = TestsFilter::from_flags(
            Some(String::new()),
            false,
            false,
            false,
            false,
            Default::default(),
            None,
        );

        let mut filtered = mocked_tests.clone();
        tests_filter.filter_tests(&mut filtered.test_cases).unwrap();

        assert_eq!(filtered.test_cases, vec![]);

        let tests_filter = TestsFilter::from_flags(
            Some("thing".to_string()),
            false,
            false,
            false,
            false,
            Default::default(),
            None,
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
            sierra_program_path: Default::default(),
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
                    },
                },
            ],
            tests_location: TestTargetLocation::Tests,
        };

        let tests_filter = TestsFilter::from_flags(
            Some(String::new()),
            true,
            false,
            false,
            false,
            Default::default(),
            None,
        );

        let mut filtered = mocked_tests.clone();
        tests_filter.filter_tests(&mut filtered.test_cases).unwrap();

        assert_eq!(filtered.test_cases, vec![]);

        let tests_filter = TestsFilter::from_flags(
            Some("thing".to_string()),
            true,
            false,
            false,
            false,
            Default::default(),
            None,
        );

        let mut filtered = mocked_tests.clone();
        tests_filter.filter_tests(&mut filtered.test_cases).unwrap();

        assert_eq!(filtered.test_cases, vec![]);

        let tests_filter = TestsFilter::from_flags(
            Some("do_thing".to_string()),
            true,
            false,
            false,
            false,
            Default::default(),
            None,
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
                },
            },]
        );

        let tests_filter = TestsFilter::from_flags(
            Some("crate1::do_thing".to_string()),
            true,
            false,
            false,
            false,
            Default::default(),
            None,
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
                },
            },]
        );

        let tests_filter = TestsFilter::from_flags(
            Some("crate3::run_other_thing".to_string()),
            true,
            false,
            false,
            false,
            Default::default(),
            None,
        );

        let mut filtered = mocked_tests.clone();
        tests_filter.filter_tests(&mut filtered.test_cases).unwrap();

        assert_eq!(filtered.test_cases, vec![]);

        let tests_filter = TestsFilter::from_flags(
            Some("outer::crate3::run_other_thing".to_string()),
            true,
            false,
            false,
            false,
            Default::default(),
            None,
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
                },
            },]
        );
    }

    #[test]
    fn filtering_with_only_ignored() {
        let mocked_tests = TestTargetWithResolvedConfig {
            sierra_program: program_for_testing(),
            sierra_program_path: Default::default(),
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
                    },
                },
            ],
            tests_location: TestTargetLocation::Tests,
        };

        let tests_filter =
            TestsFilter::from_flags(None, false, true, false, false, Default::default(), None);
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
            sierra_program_path: Default::default(),
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
                    },
                },
            ],
            tests_location: TestTargetLocation::Tests,
        };

        let tests_filter =
            TestsFilter::from_flags(None, false, false, true, false, Default::default(), None);
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
                    },
                },
            ]
        );
    }
}
