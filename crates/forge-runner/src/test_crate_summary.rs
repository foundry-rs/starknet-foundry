use crate::test_case_summary::AnyTestCaseSummary;
use crate::RunnerStatus;

/// Summary of the test run in the file
#[derive(Debug)]
pub struct TestCrateSummary {
    /// Summaries of each test case in the file
    pub test_case_summaries: Vec<AnyTestCaseSummary>,
    /// Status of the runner after executing tests in the file
    pub runner_exit_status: RunnerStatus,
}

impl TestCrateSummary {
    #[must_use]
    pub fn count_passed(&self) -> usize {
        self.test_case_summaries
            .iter()
            .filter(|tu| tu.is_passed())
            .count()
    }

    #[must_use]
    pub fn count_failed(&self) -> usize {
        self.test_case_summaries
            .iter()
            .filter(|tu| tu.is_failed())
            .count()
    }

    #[must_use]
    pub fn count_skipped(&self) -> usize {
        self.test_case_summaries
            .iter()
            .filter(|tu| tu.is_skipped())
            .count()
    }

    #[must_use]
    pub fn count_ignored(&self) -> usize {
        self.test_case_summaries
            .iter()
            .filter(|tu| tu.is_ignored())
            .count()
    }
}
