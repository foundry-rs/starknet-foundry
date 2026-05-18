use crate::test_target_summary::TestTargetSummary;
use serde::Serialize;

#[derive(Serialize)]
pub struct TestsSummary {
    passed: usize,
    failed: usize,
    interrupted: usize,
    ignored: usize,
    filtered: usize,
}

impl TestsSummary {
    #[must_use]
    pub fn new(summaries: &[TestTargetSummary], filtered: usize) -> Self {
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
        let interrupted = if self.interrupted > 0 {
            format!("\nInterrupted execution of {} test(s).", self.interrupted)
        } else {
            String::new()
        };

        format!(
            "{} passed, {} failed, {} ignored, {} filtered out{interrupted}",
            self.passed, self.failed, self.ignored, self.filtered
        )
    }
}
