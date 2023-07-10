use cairo_lang_runner::short_string::as_cairo_short_string;
use cairo_lang_runner::{RunResult, RunResultValue};
use std::{iter::Chain, option::Option, slice::Iter};
use test_collector::TestUnit;

#[derive(Debug, PartialEq, Clone)]
pub enum TestUnitSummary {
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

impl TestUnitSummary {
    pub fn from_run_result(run_result: RunResult, test_unit: &TestUnit) -> Self {
        match run_result.value {
            RunResultValue::Success(_) => TestUnitSummary::Passed {
                name: test_unit.name.to_string(),
                msg: extract_result_data(&run_result),
                run_result: Some(run_result),
            },
            RunResultValue::Panic(_) => TestUnitSummary::Failed {
                name: test_unit.name.to_string(),
                msg: extract_result_data(&run_result),
                run_result: Some(run_result),
            },
        }
    }

    pub fn skipped(test_unit: &TestUnit) -> Self {
        Self::Skipped {
            name: test_unit.name.to_string(),
        }
    }
}

pub fn extract_result_data(run_result: &RunResult) -> Option<String> {
    let data = match &run_result.value {
        RunResultValue::Success(data) => data,
        RunResultValue::Panic(data) => data,
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

#[derive(Default)]
pub struct TestSummary {
    pub passed: Vec<TestUnitSummary>,
    pub failed: Vec<TestUnitSummary>,
    pub skipped: Vec<TestUnitSummary>,
}

impl TestSummary {
    pub fn update(&mut self, test_result: TestUnitSummary) {
        match test_result {
            TestUnitSummary::Passed { .. } => self.passed.push(test_result),
            TestUnitSummary::Failed { .. } => self.failed.push(test_result),
            TestUnitSummary::Skipped { .. } => self.skipped.push(test_result),
        }
    }

    pub fn all(
        &self,
    ) -> Chain<Chain<Iter<'_, TestUnitSummary>, Iter<'_, TestUnitSummary>>, Iter<'_, TestUnitSummary>>
    {
        self.passed
            .iter()
            .chain(self.failed.iter())
            .chain(self.skipped.iter())
    }
}
