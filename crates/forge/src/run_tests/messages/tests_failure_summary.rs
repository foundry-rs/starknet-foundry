use console::style;
use forge_runner::test_case_summary::AnyTestCaseSummary;
use foundry_ui::Message;
use serde::Serialize;
use serde_json::{Value, json};
use std::fmt::Write;

#[derive(Serialize)]
pub struct TestsFailureSummaryMessage {
    pub failed_test_names: Vec<String>,
}

impl TestsFailureSummaryMessage {
    #[must_use]
    pub fn new(all_failed_tests: &[AnyTestCaseSummary]) -> Self {
        let failed_test_names = all_failed_tests
            .iter()
            .map(|any_test_case_summary| any_test_case_summary.name().unwrap().to_string())
            .collect();

        Self { failed_test_names }
    }
}

impl Message for TestsFailureSummaryMessage {
    fn text(&self) -> String {
        if self.failed_test_names.is_empty() {
            return String::new();
        }

        let mut failures = "\nFailures:".to_string();
        for name in &self.failed_test_names {
            let _ = write!(&mut failures, "\n    {name}");
        }
        style(failures).bold().to_string()
    }

    fn json(&self) -> Value {
        json!(self)
    }
}
