use cairo_felt::Felt252;
use cairo_lang_runner::short_string::as_cairo_short_string;
use cairo_lang_runner::{RunResult, RunResultValue};
use std::option::Option;
use test_collector::{ExpectedPanicValue, ExpectedTestResult, TestCase};

/// Summary of running a single test case
#[derive(Debug, PartialEq, Clone)]
pub enum TestCaseSummary {
    /// Test case passed
    Passed {
        /// Name of the test case
        name: String,
        /// Values returned by the test case run
        run_result: RunResult,
        /// Message to be printed after the test case run
        msg: Option<String>,
        /// Arguments used in the test case run
        arguments: Vec<Felt252>,
    },
    /// Test case failed
    Failed {
        /// Name of the test case
        name: String,
        /// Values returned by the test case run
        run_result: Option<RunResult>,
        /// Message returned by the test case run
        msg: Option<String>,
        /// Arguments used in the test case run
        arguments: Vec<Felt252>,
    },
    /// Test case skipped (did not run)
    Skipped {
        /// Name of the test case
        name: String,
    },
}

impl TestCaseSummary {
    pub(crate) fn arguments(&self) -> Vec<Felt252> {
        match self {
            TestCaseSummary::Failed { arguments, .. }
            | TestCaseSummary::Passed { arguments, .. } => arguments.clone(),
            TestCaseSummary::Skipped { .. } => vec![],
        }
    }
}

impl TestCaseSummary {
    #[must_use]
    pub(crate) fn from_run_result(
        run_result: RunResult,
        test_case: &TestCase,
        arguments: Vec<Felt252>,
    ) -> Self {
        let name = test_case.name.to_string();
        let msg = extract_result_data(&run_result, &test_case.expected_result);
        match run_result.clone().value {
            RunResultValue::Success(_) => match &test_case.expected_result {
                ExpectedTestResult::Success => TestCaseSummary::Passed {
                    name,
                    msg,
                    run_result,
                    arguments,
                },
                ExpectedTestResult::Panics(_) => TestCaseSummary::Failed {
                    name,
                    msg,
                    run_result: Some(run_result),
                    arguments,
                },
            },
            RunResultValue::Panic(value) => match &test_case.expected_result {
                ExpectedTestResult::Success => TestCaseSummary::Failed {
                    name,
                    msg,
                    run_result: Some(run_result),
                    arguments,
                },
                ExpectedTestResult::Panics(panic_expectation) => match panic_expectation {
                    ExpectedPanicValue::Exact(expected) if &value != expected => {
                        TestCaseSummary::Failed {
                            name,
                            msg,
                            run_result: Some(run_result),
                            arguments,
                        }
                    }
                    _ => TestCaseSummary::Passed {
                        name,
                        msg,
                        run_result,
                        arguments,
                    },
                },
            },
        }
    }

    #[must_use]
    pub(crate) fn skipped(test_case: &TestCase) -> Self {
        Self::Skipped {
            name: test_case.name.to_string(),
        }
    }
}

/// Helper function to build `readable_text` from a run data.
fn build_readable_text(data: &Vec<Felt252>) -> Option<String> {
    let mut readable_text = String::new();

    for felt in data {
        readable_text.push_str(&format!("\n    original value: [{felt}]"));
        if let Some(short_string) = as_cairo_short_string(felt) {
            readable_text.push_str(&format!(", converted to a string: [{short_string}]"));
        }
    }

    if readable_text.is_empty() {
        None
    } else {
        readable_text.push('\n');
        Some(readable_text)
    }
}

#[must_use]
/// Returns a string with the data that was produced by the test case.
/// If the test was expected to fail with specific data e.g. `#[should_panic(expected: ('data',))]`
/// and failed to do so, it returns a string comparing the panic data and the expected data.
pub(crate) fn extract_result_data(
    run_result: &RunResult,
    expectation: &ExpectedTestResult,
) -> Option<String> {
    match &run_result.value {
        RunResultValue::Success(data) => build_readable_text(data),
        RunResultValue::Panic(panic_data) => {
            let expected_data = match expectation {
                ExpectedTestResult::Panics(panic_expectation) => match panic_expectation {
                    ExpectedPanicValue::Exact(data) => Some(data),
                    ExpectedPanicValue::Any => None,
                },
                ExpectedTestResult::Success => None,
            };

            let panic_string: String = panic_data
                .iter()
                .map(|felt| as_cairo_short_string(felt).unwrap_or_default())
                .collect::<Vec<String>>()
                .join(", ");

            match expected_data {
                Some(expected) if expected == panic_data => None,
                Some(expected) => {
                    let expected_string = expected
                        .iter()
                        .map(|felt| as_cairo_short_string(felt).unwrap_or_default())
                        .collect::<Vec<String>>()
                        .join(", ");

                    Some(format!(
                        "\n    Incorrect panic data\n    {}\n    {}\n",
                        format_args!("Actual:    {panic_data:?} ({panic_string})"),
                        format_args!("Expected:  {expected:?} ({expected_string})")
                    ))
                }
                None => build_readable_text(panic_data),
            }
        }
    }
}
