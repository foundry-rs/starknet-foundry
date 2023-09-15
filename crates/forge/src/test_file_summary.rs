use crate::test_case_summary::TestCaseSummary;
use crate::RunnerStatus;
use camino::Utf8PathBuf;

/// Summary of the test run in the file
#[derive(Debug, PartialEq, Clone)]
pub struct TestFileSummary {
    /// Summaries of each test case in the file
    pub test_case_summaries: Vec<TestCaseSummary>,
    /// Status of the runner after executing tests in the file
    pub runner_exit_status: RunnerStatus,
    /// Relative path to the test file
    pub relative_path: Utf8PathBuf,
}

impl TestFileSummary {
    pub(crate) fn count_passed(&self) -> usize {
        self.test_case_summaries
            .iter()
            .filter(|tu| matches!(tu, TestCaseSummary::Passed { .. }))
            .count()
    }

    pub(crate) fn count_failed(&self) -> usize {
        self.test_case_summaries
            .iter()
            .filter(|tu| matches!(tu, TestCaseSummary::Failed { .. }))
            .count()
    }

    pub(crate) fn count_skipped(&self) -> usize {
        self.test_case_summaries
            .iter()
            .filter(|tu| matches!(tu, TestCaseSummary::Skipped { .. }))
            .count()
    }
}
