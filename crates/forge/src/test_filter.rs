use crate::compiled_raw::CompiledTestCrateRaw;
use crate::shared_cache::cached_failed_tests_names;
use anyhow::Result;
use camino::Utf8PathBuf;
use forge_runner::compiled_runnable::TestCaseRunnable;
use forge_runner::TestCaseFilter;

#[derive(Debug, PartialEq)]
// Specifies what tests should be included
pub struct TestsFilter {
    // based on name
    name_filter: NameFilter,
    // based on `#[ignore]` attribute
    ignored_filter: IgnoredFilter,
    // based on rerun_failed flag
    last_failed_filter: bool,

    cache_dir_path: Utf8PathBuf,
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
    #[allow(clippy::fn_params_excessive_bools)]
    pub fn from_flags(
        test_name_filter: Option<String>,
        exact_match: bool,
        only_ignored: bool,
        include_ignored: bool,
        rerun_failed: bool,
        cache_dir_path: Utf8PathBuf,
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
            cache_dir_path,
        }
    }

    pub(crate) fn filter_tests(
        &self,
        test_crate: CompiledTestCrateRaw,
    ) -> Result<CompiledTestCrateRaw> {
        let mut cases = test_crate.test_cases;

        cases = match &self.name_filter {
            NameFilter::All => cases,
            NameFilter::Match(filter) => cases
                .into_iter()
                .filter(|tc| tc.name.contains(filter))
                .collect(),
            NameFilter::ExactMatch(name) => {
                cases.into_iter().filter(|tc| tc.name == *name).collect()
            }
        };

        if self.last_failed_filter {
            cases = match cached_failed_tests_names(&self.cache_dir_path)? {
                Some(result) => cases
                    .into_iter()
                    .filter(|tc| result.iter().any(|name| name == &tc.name))
                    .collect(),
                None => cases,
            }
        }

        cases = match self.ignored_filter {
            // if NotIgnored (default) we filter ignored tests later and display them as ignored
            IgnoredFilter::All | IgnoredFilter::NotIgnored => cases,
            IgnoredFilter::Ignored => cases.into_iter().filter(|tc| tc.ignored).collect(),
        };

        Ok(CompiledTestCrateRaw {
            test_cases: cases,
            ..test_crate
        })
    }
}

impl TestCaseFilter for TestsFilter {
    fn should_be_run(&self, test_case: &TestCaseRunnable) -> bool {
        match self.ignored_filter {
            IgnoredFilter::All => true,
            IgnoredFilter::Ignored => test_case.ignored,
            IgnoredFilter::NotIgnored => !test_case.ignored,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::compiled_raw::{CompiledTestCrateRaw, CrateLocation, TestCaseRaw};
    use crate::test_filter::TestsFilter;
    use cairo_lang_sierra::program::Program;
    use forge_runner::expected_result::ExpectedTestResult;

    fn program_for_testing() -> Program {
        Program {
            type_declarations: vec![],
            libfunc_declarations: vec![],
            statements: vec![],
            funcs: vec![],
        }
    }

    #[test]
    #[should_panic(expected = "Arguments only_ignored and include_ignored cannot be both true")]
    fn from_flags_only_ignored_and_include_ignored_both_true() {
        let _ = TestsFilter::from_flags(None, false, true, true, false, Default::default());
    }

    #[test]
    #[should_panic(expected = "Argument test_name_filter cannot be None with exact_match")]
    fn from_flags_exact_match_true_without_test_filter_name() {
        let _ = TestsFilter::from_flags(None, true, false, false, false, Default::default());
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn filtering_tests() {
        let mocked_tests = CompiledTestCrateRaw {
            sierra_program: program_for_testing(),
            test_cases: vec![
                TestCaseRaw {
                    name: "crate1::do_thing".to_string(),
                    available_gas: None,
                    ignored: false,
                    expected_result: ExpectedTestResult::Success,
                    fork_config: None,
                    fuzzer_config: None,
                },
                TestCaseRaw {
                    name: "crate2::run_other_thing".to_string(),
                    available_gas: None,
                    ignored: true,
                    expected_result: ExpectedTestResult::Success,
                    fork_config: None,
                    fuzzer_config: None,
                },
                TestCaseRaw {
                    name: "outer::crate2::execute_next_thing".to_string(),
                    available_gas: None,
                    ignored: true,
                    expected_result: ExpectedTestResult::Success,
                    fork_config: None,
                    fuzzer_config: None,
                },
                TestCaseRaw {
                    name: "thing".to_string(),
                    available_gas: None,
                    ignored: false,
                    expected_result: ExpectedTestResult::Success,
                    fork_config: None,
                    fuzzer_config: None,
                },
            ],
            tests_location: CrateLocation::Lib,
        };

        let tests_filter = TestsFilter::from_flags(
            Some("do".to_string()),
            false,
            false,
            false,
            false,
            Default::default(),
        );
        let filtered = tests_filter.filter_tests(mocked_tests.clone()).unwrap();
        assert_eq!(
            filtered.test_cases,
            vec![TestCaseRaw {
                name: "crate1::do_thing".to_string(),
                available_gas: None,
                ignored: false,
                expected_result: ExpectedTestResult::Success,
                fork_config: None,
                fuzzer_config: None
            },]
        );

        let tests_filter = TestsFilter::from_flags(
            Some("te2::run".to_string()),
            false,
            false,
            false,
            false,
            Default::default(),
        );
        let filtered = tests_filter.filter_tests(mocked_tests.clone()).unwrap();
        assert_eq!(
            filtered.test_cases,
            vec![TestCaseRaw {
                name: "crate2::run_other_thing".to_string(),
                available_gas: None,
                ignored: true,
                expected_result: ExpectedTestResult::Success,
                fork_config: None,
                fuzzer_config: None
            },]
        );

        let tests_filter = TestsFilter::from_flags(
            Some("thing".to_string()),
            false,
            false,
            false,
            false,
            Default::default(),
        );
        let filtered = tests_filter.filter_tests(mocked_tests.clone()).unwrap();
        assert_eq!(
            filtered.test_cases,
            vec![
                TestCaseRaw {
                    name: "crate1::do_thing".to_string(),
                    available_gas: None,
                    expected_result: ExpectedTestResult::Success,
                    ignored: false,
                    fork_config: None,
                    fuzzer_config: None,
                },
                TestCaseRaw {
                    name: "crate2::run_other_thing".to_string(),
                    available_gas: None,
                    expected_result: ExpectedTestResult::Success,
                    ignored: true,
                    fork_config: None,
                    fuzzer_config: None,
                },
                TestCaseRaw {
                    name: "outer::crate2::execute_next_thing".to_string(),
                    available_gas: None,
                    expected_result: ExpectedTestResult::Success,
                    ignored: true,
                    fork_config: None,
                    fuzzer_config: None,
                },
                TestCaseRaw {
                    name: "thing".to_string(),
                    available_gas: None,
                    ignored: false,
                    expected_result: ExpectedTestResult::Success,
                    fork_config: None,
                    fuzzer_config: None,
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
        );
        let filtered = tests_filter.filter_tests(mocked_tests.clone()).unwrap();
        assert_eq!(filtered.test_cases, vec![]);

        let tests_filter = TestsFilter::from_flags(
            Some(String::new()),
            false,
            false,
            false,
            false,
            Default::default(),
        );
        let filtered = tests_filter.filter_tests(mocked_tests.clone()).unwrap();
        assert_eq!(
            filtered.test_cases,
            vec![
                TestCaseRaw {
                    name: "crate1::do_thing".to_string(),
                    available_gas: None,
                    ignored: false,
                    expected_result: ExpectedTestResult::Success,
                    fork_config: None,
                    fuzzer_config: None,
                },
                TestCaseRaw {
                    name: "crate2::run_other_thing".to_string(),
                    available_gas: None,
                    ignored: true,
                    expected_result: ExpectedTestResult::Success,
                    fork_config: None,
                    fuzzer_config: None,
                },
                TestCaseRaw {
                    name: "outer::crate2::execute_next_thing".to_string(),
                    available_gas: None,
                    ignored: true,
                    expected_result: ExpectedTestResult::Success,
                    fork_config: None,
                    fuzzer_config: None,
                },
                TestCaseRaw {
                    name: "thing".to_string(),
                    available_gas: None,
                    ignored: false,
                    expected_result: ExpectedTestResult::Success,
                    fork_config: None,
                    fuzzer_config: None,
                },
            ]
        );
    }

    #[test]
    fn filtering_with_no_tests() {
        let mocked_tests = CompiledTestCrateRaw {
            sierra_program: program_for_testing(),
            test_cases: vec![],
            tests_location: CrateLocation::Lib,
        };

        let tests_filter = TestsFilter::from_flags(
            Some(String::new()),
            false,
            false,
            false,
            false,
            Default::default(),
        );
        let filtered = tests_filter.filter_tests(mocked_tests.clone()).unwrap();
        assert_eq!(filtered.test_cases, vec![]);

        let tests_filter = TestsFilter::from_flags(
            Some("thing".to_string()),
            false,
            false,
            false,
            false,
            Default::default(),
        );
        let filtered = tests_filter.filter_tests(mocked_tests.clone()).unwrap();
        assert_eq!(filtered.test_cases, vec![]);
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn filtering_with_exact_match() {
        let mocked_tests = CompiledTestCrateRaw {
            sierra_program: program_for_testing(),
            test_cases: vec![
                TestCaseRaw {
                    name: "crate1::do_thing".to_string(),
                    available_gas: None,
                    ignored: false,
                    expected_result: ExpectedTestResult::Success,
                    fork_config: None,
                    fuzzer_config: None,
                },
                TestCaseRaw {
                    name: "crate2::run_other_thing".to_string(),
                    available_gas: None,
                    ignored: true,
                    expected_result: ExpectedTestResult::Success,
                    fork_config: None,
                    fuzzer_config: None,
                },
                TestCaseRaw {
                    name: "outer::crate3::run_other_thing".to_string(),
                    available_gas: None,
                    ignored: true,
                    expected_result: ExpectedTestResult::Success,
                    fork_config: None,
                    fuzzer_config: None,
                },
                TestCaseRaw {
                    name: "do_thing".to_string(),
                    available_gas: None,
                    ignored: false,
                    expected_result: ExpectedTestResult::Success,
                    fork_config: None,
                    fuzzer_config: None,
                },
            ],
            tests_location: CrateLocation::Tests,
        };

        let tests_filter = TestsFilter::from_flags(
            Some(String::new()),
            true,
            false,
            false,
            false,
            Default::default(),
        );
        let filtered = tests_filter.filter_tests(mocked_tests.clone()).unwrap();
        assert_eq!(filtered.test_cases, vec![]);

        let tests_filter = TestsFilter::from_flags(
            Some("thing".to_string()),
            true,
            false,
            false,
            false,
            Default::default(),
        );
        let filtered = tests_filter.filter_tests(mocked_tests.clone()).unwrap();
        assert_eq!(filtered.test_cases, vec![]);

        let tests_filter = TestsFilter::from_flags(
            Some("do_thing".to_string()),
            true,
            false,
            false,
            false,
            Default::default(),
        );
        let filtered = tests_filter.filter_tests(mocked_tests.clone()).unwrap();
        assert_eq!(
            filtered.test_cases,
            vec![TestCaseRaw {
                name: "do_thing".to_string(),
                available_gas: None,
                ignored: false,
                expected_result: ExpectedTestResult::Success,
                fork_config: None,
                fuzzer_config: None,
            },]
        );

        let tests_filter = TestsFilter::from_flags(
            Some("crate1::do_thing".to_string()),
            true,
            false,
            false,
            false,
            Default::default(),
        );
        let filtered = tests_filter.filter_tests(mocked_tests.clone()).unwrap();
        assert_eq!(
            filtered.test_cases,
            vec![TestCaseRaw {
                name: "crate1::do_thing".to_string(),
                available_gas: None,
                ignored: false,
                expected_result: ExpectedTestResult::Success,
                fork_config: None,
                fuzzer_config: None,
            },]
        );

        let tests_filter = TestsFilter::from_flags(
            Some("crate3::run_other_thing".to_string()),
            true,
            false,
            false,
            false,
            Default::default(),
        );
        let filtered = tests_filter.filter_tests(mocked_tests.clone()).unwrap();
        assert_eq!(filtered.test_cases, vec![]);

        let tests_filter = TestsFilter::from_flags(
            Some("outer::crate3::run_other_thing".to_string()),
            true,
            false,
            false,
            false,
            Default::default(),
        );
        let filtered = tests_filter.filter_tests(mocked_tests.clone()).unwrap();
        assert_eq!(
            filtered.test_cases,
            vec![TestCaseRaw {
                name: "outer::crate3::run_other_thing".to_string(),
                available_gas: None,
                ignored: true,
                expected_result: ExpectedTestResult::Success,
                fork_config: None,
                fuzzer_config: None,
            },]
        );
    }

    #[test]
    fn filtering_with_only_ignored() {
        let mocked_tests = CompiledTestCrateRaw {
            sierra_program: program_for_testing(),
            test_cases: vec![
                TestCaseRaw {
                    name: "crate1::do_thing".to_string(),
                    available_gas: None,
                    ignored: false,
                    expected_result: ExpectedTestResult::Success,
                    fork_config: None,
                    fuzzer_config: None,
                },
                TestCaseRaw {
                    name: "crate2::run_other_thing".to_string(),
                    available_gas: None,
                    ignored: true,
                    expected_result: ExpectedTestResult::Success,
                    fork_config: None,
                    fuzzer_config: None,
                },
                TestCaseRaw {
                    name: "outer::crate3::run_other_thing".to_string(),
                    available_gas: None,
                    ignored: true,
                    expected_result: ExpectedTestResult::Success,
                    fork_config: None,
                    fuzzer_config: None,
                },
                TestCaseRaw {
                    name: "do_thing".to_string(),
                    available_gas: None,
                    ignored: false,
                    expected_result: ExpectedTestResult::Success,
                    fork_config: None,
                    fuzzer_config: None,
                },
            ],
            tests_location: CrateLocation::Tests,
        };

        let tests_filter =
            TestsFilter::from_flags(None, false, true, false, false, Default::default());
        let filtered = tests_filter.filter_tests(mocked_tests).unwrap();
        assert_eq!(
            filtered.test_cases,
            vec![
                TestCaseRaw {
                    name: "crate2::run_other_thing".to_string(),
                    available_gas: None,
                    ignored: true,
                    expected_result: ExpectedTestResult::Success,
                    fork_config: None,
                    fuzzer_config: None,
                },
                TestCaseRaw {
                    name: "outer::crate3::run_other_thing".to_string(),
                    available_gas: None,
                    ignored: true,
                    expected_result: ExpectedTestResult::Success,
                    fork_config: None,
                    fuzzer_config: None,
                },
            ]
        );
    }

    #[test]
    fn filtering_with_include_ignored() {
        let mocked_tests = CompiledTestCrateRaw {
            sierra_program: program_for_testing(),
            test_cases: vec![
                TestCaseRaw {
                    name: "crate1::do_thing".to_string(),
                    available_gas: None,
                    ignored: false,
                    expected_result: ExpectedTestResult::Success,
                    fork_config: None,
                    fuzzer_config: None,
                },
                TestCaseRaw {
                    name: "crate2::run_other_thing".to_string(),
                    available_gas: None,
                    ignored: true,
                    expected_result: ExpectedTestResult::Success,
                    fork_config: None,
                    fuzzer_config: None,
                },
                TestCaseRaw {
                    name: "outer::crate3::run_other_thing".to_string(),
                    available_gas: None,
                    ignored: true,
                    expected_result: ExpectedTestResult::Success,
                    fork_config: None,
                    fuzzer_config: None,
                },
                TestCaseRaw {
                    name: "do_thing".to_string(),
                    available_gas: None,
                    ignored: false,
                    expected_result: ExpectedTestResult::Success,
                    fork_config: None,
                    fuzzer_config: None,
                },
            ],
            tests_location: CrateLocation::Tests,
        };

        let tests_filter =
            TestsFilter::from_flags(None, false, false, true, false, Default::default());
        let filtered = tests_filter.filter_tests(mocked_tests).unwrap();
        assert_eq!(
            filtered.test_cases,
            vec![
                TestCaseRaw {
                    name: "crate1::do_thing".to_string(),
                    available_gas: None,
                    ignored: false,
                    expected_result: ExpectedTestResult::Success,
                    fork_config: None,
                    fuzzer_config: None,
                },
                TestCaseRaw {
                    name: "crate2::run_other_thing".to_string(),
                    available_gas: None,
                    ignored: true,
                    expected_result: ExpectedTestResult::Success,
                    fork_config: None,
                    fuzzer_config: None,
                },
                TestCaseRaw {
                    name: "outer::crate3::run_other_thing".to_string(),
                    available_gas: None,
                    ignored: true,
                    expected_result: ExpectedTestResult::Success,
                    fork_config: None,
                    fuzzer_config: None,
                },
                TestCaseRaw {
                    name: "do_thing".to_string(),
                    available_gas: None,
                    ignored: false,
                    expected_result: ExpectedTestResult::Success,
                    fork_config: None,
                    fuzzer_config: None,
                },
            ]
        );
    }
}
