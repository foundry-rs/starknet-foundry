use crate::test_target_summary::TestTargetSummary;
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum FilteredTestsCount {
    Exact(usize),
    Other,
}

#[derive(Serialize)]
pub struct TestsSummary {
    passed: usize,
    failed: usize,
    interrupted: usize,
    ignored: usize,
    filtered: FilteredTestsCount,
}

impl TestsSummary {
    #[must_use]
    pub fn new(summaries: &[TestTargetSummary], filtered: FilteredTestsCount) -> Self {
        let passed = summaries.iter().map(TestTargetSummary::count_passed).sum();
        let failed = summaries.iter().map(TestTargetSummary::count_failed).sum();
        let interrupted = summaries
            .iter()
            .map(TestTargetSummary::count_interrupted)
            .sum();
        let ignored = summaries.iter().map(TestTargetSummary::count_ignored).sum();

        Self {
            passed,
            failed,
            interrupted,
            ignored,
            filtered,
        }
    }

    #[must_use]
    pub fn format_summary_message(&self) -> String {
        let filtered = match self.filtered {
            FilteredTestsCount::Exact(value) => value.to_string(),
            FilteredTestsCount::Other => "other".to_string(),
        };

        let interrupted = if self.interrupted > 0 {
            format!("\nInterrupted execution of {} test(s).", self.interrupted)
        } else {
            String::new()
        };

        format!(
            "{} passed, {} failed, {} ignored, {filtered} filtered out{interrupted}",
            self.passed, self.failed, self.ignored,
        )
    }
}
