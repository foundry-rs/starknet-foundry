use crate::test_case_summary::{TestCaseSummary, Fuzzing, Single};
use crate::RunnerStatus;

#[derive(Debug)]
pub enum AnyTestCaseSummary {
    Fuzzing(TestCaseSummary<Fuzzing>),
    Single(TestCaseSummary<Single>),
}
/// Summary of the test run in the file
#[derive(Debug)]
pub struct TestCrateSummary {
    /// Summaries of each test case in the file
    pub test_case_summaries: Vec<AnyTestCaseSummary>,
    /// Status of the runner after executing tests in the file
    pub runner_exit_status: RunnerStatus,
    /// If test crate contained fuzzed tests
    pub contained_fuzzed_tests: bool,
}

impl TestCrateSummary {
    #[must_use]
    pub fn count_passed(&self) -> usize {
        self.test_case_summaries
            .iter()
            .filter(|tu| matches!(tu, AnyTestCaseSummary::Single(TestCaseSummary::Passed { .. }) | AnyTestCaseSummary::Fuzzing(TestCaseSummary::Passed { .. }) ))
            .count()
    }

    #[must_use]
    pub fn count_failed(&self) -> usize {
        self.test_case_summaries
            .iter()
            .filter(|tu| matches!(tu, AnyTestCaseSummary::Single(TestCaseSummary::Failed { .. }) | AnyTestCaseSummary::Fuzzing(TestCaseSummary::Failed { .. }) ))
            .count()
    }

    #[must_use]
    pub fn count_skipped(&self) -> usize {
        self.test_case_summaries
            .iter()
            .filter(|tu| matches!(tu, AnyTestCaseSummary::Single(TestCaseSummary::Skipped { .. }) | AnyTestCaseSummary::Fuzzing(TestCaseSummary::Skipped { .. }) ))
            .count()
    }

    #[must_use]
    pub fn count_ignored(&self) -> usize {
        self.test_case_summaries
            .iter()
            .filter(|tu| matches!(tu, AnyTestCaseSummary::Single(TestCaseSummary::Ignored { .. }) | AnyTestCaseSummary::Fuzzing(TestCaseSummary::Ignored { .. }) ))
            .count()
    }
}
