use cairo_lang_runner::short_string::as_cairo_short_string;
use cairo_lang_runner::{RunResult, RunResultValue};
use std::option::Option;
use test_collector::TestCase;

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
    pub fn skipped(test_case: &TestCase) -> Self {
        Self::Skipped {
            name: test_case.name.to_string(),
        }
    }

    #[must_use]
    pub fn name(&self) -> &String {
        match self {
            TestCaseSummary::Passed {
                name,
                run_result: _,
                msg: _,
            }
            | TestCaseSummary::Skipped { name }
            | TestCaseSummary::Failed {
                name,
                run_result: _,
                msg: _,
            } => name,
        }
    }

    pub fn update_name(&mut self, new_name: String) {
        match self {
            TestCaseSummary::Passed {
                name,
                run_result: _,
                msg: _,
            }
            | TestCaseSummary::Skipped { name }
            | TestCaseSummary::Failed {
                name,
                run_result: _,
                msg: _,
            } => {
                *name = new_name;
            }
        }
    }
}

#[must_use]
pub fn extract_result_data(run_result: &RunResult) -> Option<String> {
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
