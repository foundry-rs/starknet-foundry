use crate::collecting::{CompiledTestCrate, CompiledTestCrateRaw, ValidatedForkConfig};
use crate::TestCaseFilter;
use test_collector::TestCase;

use crate::shared_cache::get_cached_failed_tests_names;

#[derive(Debug, PartialEq)]
// Specifies what tests should be included
pub struct TestsFilter {
    // based on names
    name_filters: Vec<NameFilter>,
    // based on `#[ignore]` attribute
    ignored_filter: IgnoredFilter,
}

#[derive(Debug, PartialEq)]
pub(crate) enum NameFilter {
    All,
    Match(String),
    ExactMatch(String),
    RerunFailed,
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
    ) -> Self {
        assert!(!(only_ignored && include_ignored));

        let ignored_filter = if include_ignored {
            IgnoredFilter::All
        } else if only_ignored {
            IgnoredFilter::Ignored
        } else {
            IgnoredFilter::NotIgnored
        };

        let mut name_filters = vec![];

        if exact_match {
            name_filters.push(NameFilter::ExactMatch(test_name_filter.unwrap()))
        } else if let Some(name) = test_name_filter {
            name_filters.push(NameFilter::Match(name))
        } else {
            name_filters.push(NameFilter::All)
        };

        if rerun_failed {
            name_filters.push(NameFilter::RerunFailed)
        }

        Self {
            name_filters,
            ignored_filter,
        }
    }

    pub(crate) fn filter_tests(&self, test_crate: CompiledTestCrateRaw) -> CompiledTestCrateRaw {
        let mut cases = test_crate.test_cases;
        for name_filter in &self.name_filters {
            cases = match name_filter {
                NameFilter::All => cases,
                NameFilter::Match(filter) => cases
                    .into_iter()
                    .filter(|tc| tc.name.contains(filter))
                    .collect(),
                NameFilter::ExactMatch(name) => {
                    cases.into_iter().filter(|tc| tc.name == *name).collect()
                }
                NameFilter::RerunFailed => {
                    match get_cached_failed_tests_names().expect("Can't read cached failed tests") {
                        Some(result) => cases
                            .into_iter()
                            .filter(|tc| result.iter().any(|name| name == &tc.name))
                            .collect(),
                        None => cases,
                    }
                }
            };
        }
        cases = match self.ignored_filter {
            // if NotIgnored (default) we filter ignored tests later and display them as ignored
            IgnoredFilter::All | IgnoredFilter::NotIgnored => cases,
            IgnoredFilter::Ignored => cases.into_iter().filter(|tc| tc.ignored).collect(),
        };

        CompiledTestCrate {
            test_cases: cases,
            ..test_crate
        }
    }
}

impl TestCaseFilter for TestsFilter {
    fn should_be_run(&self, test_case: &TestCase<ValidatedForkConfig>) -> bool {
        match self.ignored_filter {
            IgnoredFilter::All => true,
            IgnoredFilter::Ignored => test_case.ignored,
            IgnoredFilter::NotIgnored => !test_case.ignored,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::collecting::CompiledTestCrate;
    use crate::test_filter::TestsFilter;
    use crate::CrateLocation;
    use cairo_lang_sierra::program::Program;
    use test_collector::{ExpectedTestResult, TestCase};

    fn program_for_testing() -> Program {
        Program {
            type_declarations: vec![],
            libfunc_declarations: vec![],
            statements: vec![],
            funcs: vec![],
        }
    }

    #[test]
    #[should_panic]
    fn from_flags_only_ignored_and_include_ignored_both_true() {
        let _ = TestsFilter::from_flags(None, false, true, true, false);
    }

    #[test]
    #[should_panic]
    fn from_flags_exact_match_true_without_test_filter_name() {
        let _ = TestsFilter::from_flags(None, true, false, false, false);
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn filtering_tests() {
        let mocked_tests = CompiledTestCrate {
            sierra_program: program_for_testing(),
            test_cases: vec![
                TestCase {
                    name: "crate1::do_thing".to_string(),
                    available_gas: None,
                    ignored: false,
                    expected_result: ExpectedTestResult::Success,
                    fork_config: None,
                    fuzzer_config: None,
                },
                TestCase {
                    name: "crate2::run_other_thing".to_string(),
                    available_gas: None,
                    ignored: true,
                    expected_result: ExpectedTestResult::Success,
                    fork_config: None,
                    fuzzer_config: None,
                },
                TestCase {
                    name: "outer::crate2::execute_next_thing".to_string(),
                    available_gas: None,
                    ignored: true,
                    expected_result: ExpectedTestResult::Success,
                    fork_config: None,
                    fuzzer_config: None,
                },
                TestCase {
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

        let tests_filter =
            TestsFilter::from_flags(Some("do".to_string()), false, false, false, false);
        let filtered = tests_filter.filter_tests(mocked_tests.clone());
        assert_eq!(
            filtered.test_cases,
            vec![TestCase {
                name: "crate1::do_thing".to_string(),
                available_gas: None,
                ignored: false,
                expected_result: ExpectedTestResult::Success,
                fork_config: None,
                fuzzer_config: None
            },]
        );

        let tests_filter =
            TestsFilter::from_flags(Some("te2::run".to_string()), false, false, false, false);
        let filtered = tests_filter.filter_tests(mocked_tests.clone());
        assert_eq!(
            filtered.test_cases,
            vec![TestCase {
                name: "crate2::run_other_thing".to_string(),
                available_gas: None,
                ignored: true,
                expected_result: ExpectedTestResult::Success,
                fork_config: None,
                fuzzer_config: None
            },]
        );

        let tests_filter =
            TestsFilter::from_flags(Some("thing".to_string()), false, false, false, false);
        let filtered = tests_filter.filter_tests(mocked_tests.clone());
        assert_eq!(
            filtered.test_cases,
            vec![
                TestCase {
                    name: "crate1::do_thing".to_string(),
                    available_gas: None,
                    expected_result: ExpectedTestResult::Success,
                    ignored: false,
                    fork_config: None,
                    fuzzer_config: None,
                },
                TestCase {
                    name: "crate2::run_other_thing".to_string(),
                    available_gas: None,
                    expected_result: ExpectedTestResult::Success,
                    ignored: true,
                    fork_config: None,
                    fuzzer_config: None,
                },
                TestCase {
                    name: "outer::crate2::execute_next_thing".to_string(),
                    available_gas: None,
                    expected_result: ExpectedTestResult::Success,
                    ignored: true,
                    fork_config: None,
                    fuzzer_config: None,
                },
                TestCase {
                    name: "thing".to_string(),
                    available_gas: None,
                    ignored: false,
                    expected_result: ExpectedTestResult::Success,
                    fork_config: None,
                    fuzzer_config: None,
                },
            ]
        );

        let tests_filter =
            TestsFilter::from_flags(Some("nonexistent".to_string()), false, false, false, false);
        let filtered = tests_filter.filter_tests(mocked_tests.clone());
        assert_eq!(filtered.test_cases, vec![]);

        let tests_filter = TestsFilter::from_flags(Some(String::new()), false, false, false, false);
        let filtered = tests_filter.filter_tests(mocked_tests.clone());
        assert_eq!(
            filtered.test_cases,
            vec![
                TestCase {
                    name: "crate1::do_thing".to_string(),
                    available_gas: None,
                    ignored: false,
                    expected_result: ExpectedTestResult::Success,
                    fork_config: None,
                    fuzzer_config: None,
                },
                TestCase {
                    name: "crate2::run_other_thing".to_string(),
                    available_gas: None,
                    ignored: true,
                    expected_result: ExpectedTestResult::Success,
                    fork_config: None,
                    fuzzer_config: None,
                },
                TestCase {
                    name: "outer::crate2::execute_next_thing".to_string(),
                    available_gas: None,
                    ignored: true,
                    expected_result: ExpectedTestResult::Success,
                    fork_config: None,
                    fuzzer_config: None,
                },
                TestCase {
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
        let mocked_tests = CompiledTestCrate {
            sierra_program: program_for_testing(),
            test_cases: vec![],
            tests_location: CrateLocation::Lib,
        };

        let tests_filter = TestsFilter::from_flags(Some(String::new()), false, false, false, false);
        let filtered = tests_filter.filter_tests(mocked_tests.clone());
        assert_eq!(filtered.test_cases, vec![]);

        let tests_filter =
            TestsFilter::from_flags(Some("thing".to_string()), false, false, false, false);
        let filtered = tests_filter.filter_tests(mocked_tests.clone());
        assert_eq!(filtered.test_cases, vec![]);
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn filtering_with_exact_match() {
        let mocked_tests = CompiledTestCrate {
            sierra_program: program_for_testing(),
            test_cases: vec![
                TestCase {
                    name: "crate1::do_thing".to_string(),
                    available_gas: None,
                    ignored: false,
                    expected_result: ExpectedTestResult::Success,
                    fork_config: None,
                    fuzzer_config: None,
                },
                TestCase {
                    name: "crate2::run_other_thing".to_string(),
                    available_gas: None,
                    ignored: true,
                    expected_result: ExpectedTestResult::Success,
                    fork_config: None,
                    fuzzer_config: None,
                },
                TestCase {
                    name: "outer::crate3::run_other_thing".to_string(),
                    available_gas: None,
                    ignored: true,
                    expected_result: ExpectedTestResult::Success,
                    fork_config: None,
                    fuzzer_config: None,
                },
                TestCase {
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

        let tests_filter = TestsFilter::from_flags(Some(String::new()), true, false, false, false);
        let filtered = tests_filter.filter_tests(mocked_tests.clone());
        assert_eq!(filtered.test_cases, vec![]);

        let tests_filter =
            TestsFilter::from_flags(Some("thing".to_string()), true, false, false, false);
        let filtered = tests_filter.filter_tests(mocked_tests.clone());
        assert_eq!(filtered.test_cases, vec![]);

        let tests_filter =
            TestsFilter::from_flags(Some("do_thing".to_string()), true, false, false, false);
        let filtered = tests_filter.filter_tests(mocked_tests.clone());
        assert_eq!(
            filtered.test_cases,
            vec![TestCase {
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
        );
        let filtered = tests_filter.filter_tests(mocked_tests.clone());
        assert_eq!(
            filtered.test_cases,
            vec![TestCase {
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
        );
        let filtered = tests_filter.filter_tests(mocked_tests.clone());
        assert_eq!(filtered.test_cases, vec![]);

        let tests_filter = TestsFilter::from_flags(
            Some("outer::crate3::run_other_thing".to_string()),
            true,
            false,
            false,
            false,
        );
        let filtered = tests_filter.filter_tests(mocked_tests.clone());
        assert_eq!(
            filtered.test_cases,
            vec![TestCase {
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
        let mocked_tests = CompiledTestCrate {
            sierra_program: program_for_testing(),
            test_cases: vec![
                TestCase {
                    name: "crate1::do_thing".to_string(),
                    available_gas: None,
                    ignored: false,
                    expected_result: ExpectedTestResult::Success,
                    fork_config: None,
                    fuzzer_config: None,
                },
                TestCase {
                    name: "crate2::run_other_thing".to_string(),
                    available_gas: None,
                    ignored: true,
                    expected_result: ExpectedTestResult::Success,
                    fork_config: None,
                    fuzzer_config: None,
                },
                TestCase {
                    name: "outer::crate3::run_other_thing".to_string(),
                    available_gas: None,
                    ignored: true,
                    expected_result: ExpectedTestResult::Success,
                    fork_config: None,
                    fuzzer_config: None,
                },
                TestCase {
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

        let tests_filter = TestsFilter::from_flags(None, false, true, false, false);
        let filtered = tests_filter.filter_tests(mocked_tests);
        assert_eq!(
            filtered.test_cases,
            vec![
                TestCase {
                    name: "crate2::run_other_thing".to_string(),
                    available_gas: None,
                    ignored: true,
                    expected_result: ExpectedTestResult::Success,
                    fork_config: None,
                    fuzzer_config: None,
                },
                TestCase {
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
        let mocked_tests = CompiledTestCrate {
            sierra_program: program_for_testing(),
            test_cases: vec![
                TestCase {
                    name: "crate1::do_thing".to_string(),
                    available_gas: None,
                    ignored: false,
                    expected_result: ExpectedTestResult::Success,
                    fork_config: None,
                    fuzzer_config: None,
                },
                TestCase {
                    name: "crate2::run_other_thing".to_string(),
                    available_gas: None,
                    ignored: true,
                    expected_result: ExpectedTestResult::Success,
                    fork_config: None,
                    fuzzer_config: None,
                },
                TestCase {
                    name: "outer::crate3::run_other_thing".to_string(),
                    available_gas: None,
                    ignored: true,
                    expected_result: ExpectedTestResult::Success,
                    fork_config: None,
                    fuzzer_config: None,
                },
                TestCase {
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

        let tests_filter = TestsFilter::from_flags(None, false, false, true, false);
        let filtered = tests_filter.filter_tests(mocked_tests);
        assert_eq!(
            filtered.test_cases,
            vec![
                TestCase {
                    name: "crate1::do_thing".to_string(),
                    available_gas: None,
                    ignored: false,
                    expected_result: ExpectedTestResult::Success,
                    fork_config: None,
                    fuzzer_config: None,
                },
                TestCase {
                    name: "crate2::run_other_thing".to_string(),
                    available_gas: None,
                    ignored: true,
                    expected_result: ExpectedTestResult::Success,
                    fork_config: None,
                    fuzzer_config: None,
                },
                TestCase {
                    name: "outer::crate3::run_other_thing".to_string(),
                    available_gas: None,
                    ignored: true,
                    expected_result: ExpectedTestResult::Success,
                    fork_config: None,
                    fuzzer_config: None,
                },
                TestCase {
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
