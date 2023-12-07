use starknet_api::block::BlockNumber;

use crate::test_case_summary::{Fuzzing, Single, TestCaseSummary};
use crate::RunnerStatus;

#[derive(Debug)]
pub enum AnyTestCaseSummary {
    Fuzzing(TestCaseSummary<Fuzzing>),
    Single(TestCaseSummary<Single>),
}

impl AnyTestCaseSummary {
    #[must_use]
    pub fn name(&self) -> Option<&String> {
        match self {
            AnyTestCaseSummary::Fuzzing(case) => case.name(),
            AnyTestCaseSummary::Single(case) => case.name(),
        }
    }

    #[must_use]
    pub fn msg(&self) -> Option<&String> {
        match self {
            AnyTestCaseSummary::Fuzzing(case) => case.msg(),
            AnyTestCaseSummary::Single(case) => case.msg(),
        }
    }

    #[must_use]
    pub fn latest_block_number(&self) -> Option<&BlockNumber> {
        match self {
            AnyTestCaseSummary::Fuzzing(case) => case.latest_block_number(),
            AnyTestCaseSummary::Single(case) => case.latest_block_number(),
        }
    }

    #[must_use]
    pub fn is_passed(&self) -> bool {
        matches!(
            self,
            AnyTestCaseSummary::Single(TestCaseSummary::Passed { .. })
                | AnyTestCaseSummary::Fuzzing(TestCaseSummary::Passed { .. })
        )
    }

    #[must_use]
    pub fn is_failed(&self) -> bool {
        matches!(
            self,
            AnyTestCaseSummary::Single(TestCaseSummary::Failed { .. })
                | AnyTestCaseSummary::Fuzzing(TestCaseSummary::Failed { .. })
        )
    }

    #[must_use]
    pub fn is_skipped(&self) -> bool {
        matches!(
            self,
            AnyTestCaseSummary::Single(TestCaseSummary::Skipped { .. })
                | AnyTestCaseSummary::Fuzzing(TestCaseSummary::Skipped { .. })
        )
    }

    #[must_use]
    pub fn is_ignored(&self) -> bool {
        matches!(
            self,
            AnyTestCaseSummary::Single(TestCaseSummary::Ignored { .. })
                | AnyTestCaseSummary::Fuzzing(TestCaseSummary::Ignored { .. })
        )
    }
}

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
