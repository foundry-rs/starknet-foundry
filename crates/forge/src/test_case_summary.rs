use cairo_lang_runner::short_string::as_cairo_short_string;
use cairo_lang_runner::{RunResult, RunResultValue};
use std::option::Option;
use test_collector::TestCase;

/// Summary of running a single test case
#[derive(Debug, PartialEq, Clone)]
pub enum TestCaseSummary {
    /// Test case passed
    Passed {
        /// Name of the test case
        name: String,
        /// Values returned by the test case run
        run_result: RunResult,
        /// Message returned by the test case run
        msg: Option<String>,
    },
    /// Test case failed
    Failed {
        /// Name of the test case
        name: String,
        /// Values returned by the test case run
        run_result: Option<RunResult>,
        /// Message returned by the test case run
        msg: Option<String>,
    },
    /// Test case skipped (did not run)
    Skipped {
        /// Name of the test case
        name: String,
    },
}

impl TestCaseSummary {
    #[must_use]
    pub(crate) fn from_run_result(run_result: RunResult, test_case: &TestCase) -> Self {
        match run_result.value {
            RunResultValue::Success(_) => TestCaseSummary::Passed {
                name: test_case.name.to_string(),
                msg: extract_result_data(&run_result),
                run_result,
            },
            RunResultValue::Panic(_) => TestCaseSummary::Failed {
                name: test_case.name.to_string(),
                msg: extract_result_data(&run_result),
                run_result: Some(run_result),
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

#[must_use]
fn extract_result_data(run_result: &RunResult) -> Option<String> {
    let data = match &run_result.value {
        RunResultValue::Panic(data) | RunResultValue::Success(data) => data,
    };

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
