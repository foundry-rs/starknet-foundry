use cairo_lang_runner::short_string::as_cairo_short_string;
use cairo_lang_runner::{RunResult, RunResultValue};
use std::option::Option;
use test_collector::{PanicExpectation, TestCase, TestExpectation};

#[derive(Debug, PartialEq, Clone)]
pub enum TestCaseSummary {
    Passed {
        name: String,
        run_result: RunResult,
        msg: Option<String>,
    },
    Failed {
        name: String,
        run_result: Option<RunResult>,
        msg: Option<String>,
    },
    Skipped {
        name: String,
    },
}

impl TestCaseSummary {
    #[must_use]
    pub fn from_run_result(run_result: RunResult, test_case: &TestCase) -> Self {
        let name = test_case.name.to_string();
        let msg = extract_result_data(&run_result, &test_case.expectation);
        match run_result.clone().value {
            RunResultValue::Success(_) => TestCaseSummary::Passed {
                name,
                msg,
                run_result,
            },
            RunResultValue::Panic(value) => match &test_case.expectation {
                TestExpectation::Success => TestCaseSummary::Failed {
                    name,
                    msg,
                    run_result: Some(run_result),
                },
                TestExpectation::Panics(panic_expectation) => match panic_expectation {
                    PanicExpectation::Exact(expected) if &value != expected => {
                        TestCaseSummary::Failed {
                            name,
                            msg,
                            run_result: Some(run_result),
                        }
                    }
                    _ => TestCaseSummary::Passed {
                        name,
                        msg,
                        run_result,
                    },
                },
            }
        }
    }

    #[must_use]
    pub fn skipped(test_case: &TestCase) -> Self {
        Self::Skipped {
            name: test_case.name.to_string(),
        }
    }
}

#[must_use]
/// Returns a string with the data that was produced by the test case.
/// If the test case was successful, it returns the data that was produced by the test case.
/// If the test case failed, it returns a string comparing the panic data and the expected data.
pub fn extract_result_data(run_result: &RunResult, expectation: &TestExpectation) -> Option<String> {
    match &run_result.value {
        RunResultValue::Success(data) => {
            let readable_text: String = data.iter()
                .map(|felt| {
                    let short_string = as_cairo_short_string(felt).map_or(String::new(), |s| format!(", converted to a string: [{:?}]", s));
                    format!("\n    original value: [{:?}]{}", felt, short_string)
                })
                .collect();

            if readable_text.is_empty() {
                None
            } else {
                Some(format!("{}{}", readable_text, '\n'))
            }
        }
        RunResultValue::Panic(panic_data) => {
            let expected_data = match expectation {
                TestExpectation::Panics(panic_expectation) => match panic_expectation {
                    PanicExpectation::Exact(data) => Some(data),
                    PanicExpectation::Any => None,
                },
                TestExpectation::Success => None,
            };

            let panic_string: Vec<String> = panic_data.iter()
                .map(|felt| {
                    as_cairo_short_string(felt).unwrap_or_default()
                })
                .collect();

            match expected_data {
                Some(expected) if expected == panic_data => None,
                Some(expected) => {
                    let expected_string: Vec<String> = expected.iter()
                        .map(|felt| {
                            as_cairo_short_string(felt).unwrap_or_default()
                        })
                        .collect();

                    Some(format!("\x1B[31mFAIL: Test did not meet expectations!\x1B[0m\n\
                                  \x1B[32m    Actual:   {:?} ({:?})\x1B[0m\n\
                                  \x1B[31m    Expected: {:?} ({:?})\x1B[0m\n",
                                 panic_data, panic_string, expected, expected_string))
                }
                None => Some(format!("\x1B[31mERROR: Run panicked with data: {:?}\x1B[0m\n", panic_data)),
            }
        }
    }
}
