use crate::test_target_summary::TestTargetSummary;
use serde::Serialize;

// TODO(#2574): Bring back "filtered out" number in tests summary when running with `--exact` flag
#[derive(Serialize)]
pub struct TestsSummary {
    passed: usize,
    failed: usize,
    interrupted: usize,
    ignored: usize,
    filtered: Option<usize>,
    skipped: Option<usize>,
}

impl TestsSummary {
    #[must_use]
    pub fn new(summaries: &[TestTargetSummary], filtered: Option<usize>, skipped: Option<usize>) -> Self {
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
            skipped
        }
    }

    #[must_use]
    pub fn format_summary_message(&self) -> String {
        let filtered = self
            .filtered
            .map_or_else(|| "other".to_string(), |v| v.to_string());

        let interrupted = if self.interrupted > 0 {
            format!("\nInterrupted execution of {} test(s).", self.interrupted)
        } else {
            String::new()
        };

        let summary = format!(
            "{} passed, {} failed, {} ignored, {filtered} filtered out{interrupted}",
            self.passed, self.failed, self.ignored,
        );

        if let Some(skipped) = self.skipped {
            let skipped_str = format!(", {} skipped", skipped);
            format!("{summary}{skipped_str}")
        } else {
            summary
        }
    }
}
