use cairo_lang_runner::short_string::as_cairo_short_string;
use cairo_lang_runner::{RunResult, RunResultValue};
use std::{iter::Chain, option::Option, slice::Iter};

pub enum TestResult {
    Passed {
        name: String,
        run_result: Option<RunResult>,
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

pub fn extract_result_data(run_result: &RunResult) -> Option<String> {
    let data = match &run_result.value {
        RunResultValue::Success(data) => data,
        RunResultValue::Panic(data) => data,
    };

    let mut readable_text = String::new();

    for felt in data {
        readable_text.push_str(&format!("\n    original value: [{felt}]"));
        if let Some(short_string) = as_cairo_short_string(&felt) {
            readable_text.push_str(&format!(", converted to a string: [{short_string}]"));
        }
    }

    if readable_text.is_empty() {
        None
    } else {
        readable_text.push_str("\n");
        Some(readable_text)
    }
}

#[derive(Default)]
pub struct TestSummary {
    pub passed: Vec<TestResult>,
    pub failed: Vec<TestResult>,
    pub skipped: Vec<TestResult>,
}

impl TestSummary {
    pub fn update(&mut self, test_result: TestResult) {
        match test_result {
            TestResult::Passed { .. } => self.passed.push(test_result),
            TestResult::Failed { .. } => self.failed.push(test_result),
            TestResult::Skipped { .. } => self.skipped.push(test_result),
        }
    }

    pub fn all(
        &self,
    ) -> Chain<Chain<Iter<'_, TestResult>, Iter<'_, TestResult>>, Iter<'_, TestResult>> {
        self.passed
            .iter()
            .chain(self.failed.iter())
            .chain(self.skipped.iter())
    }
}
