use cairo_lang_runner::short_string::as_cairo_short_string;
use cairo_lang_runner::{RunResult, RunResultValue};
use std::option::Option;
use test_collector::TestUnit;

#[derive(Debug, PartialEq, Clone)]
pub enum TestUnitSummary {
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

impl TestUnitSummary {
    #[must_use]
    pub fn from_run_result(run_result: RunResult, test_unit: &TestUnit) -> Self {
        match run_result.value {
            RunResultValue::Success(_) => TestUnitSummary::Passed {
                name: test_unit.name.to_string(),
                msg: extract_result_data(&run_result),
                run_result,
            },
            RunResultValue::Panic(_) => TestUnitSummary::Failed {
                name: test_unit.name.to_string(),
                msg: extract_result_data(&run_result),
                run_result: Some(run_result),
            },
        }
    }

    #[must_use]
    pub fn skipped(test_unit: &TestUnit) -> Self {
        Self::Skipped {
            name: test_unit.name.to_string(),
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
