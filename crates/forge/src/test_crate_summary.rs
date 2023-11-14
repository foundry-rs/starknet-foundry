use crate::test_case_summary::TestCaseSummary;
use crate::{CrateLocation, RunnerStatus};

/// Summary of the test run in the file
#[derive(Debug, PartialEq)]
pub struct TestCrateSummary {
    /// Summaries of each test case in the file
    pub test_case_summaries: Vec<TestCaseSummary>,
    /// Status of the runner after executing tests in the file
    pub runner_exit_status: RunnerStatus,
    /// Type of the test crate
    pub test_crate_type: CrateLocation,
    /// If test crate contained fuzzed tests
    pub contained_fuzzed_tests: bool,
}

impl TestCrateSummary {
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

    pub(crate) fn count_ignored(&self) -> usize {
        self.test_case_summaries
            .iter()
            .filter(|tu| matches!(tu, TestCaseSummary::Ignored { .. }))
            .count()
    }
}
