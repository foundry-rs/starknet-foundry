use crate::test_case_summary::AnyTestCaseSummary;

/// Summary of the test run in the file
#[derive(Debug)]
pub struct TestTargetSummary {
    /// Summaries of each test case in the file
    pub test_case_summaries: Vec<AnyTestCaseSummary>,
}

impl TestTargetSummary {
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
    pub fn count_interrupted(&self) -> usize {
        self.test_case_summaries
            .iter()
            .filter(|tu| tu.is_interrupted())
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
