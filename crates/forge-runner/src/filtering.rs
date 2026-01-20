use crate::package_tests::TestCase;

/// Result of filtering a test case.
#[derive(Debug)]
pub enum FilterResult {
    /// Test case should be included.
    Included,
    /// Test case should be excluded for the given reason.
    Excluded(ExcludeReason),
}

/// Reason for excluding a test case.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExcludeReason {
    /// Test case is excluded by the ignore filter.
    ExcludedByIgnoreFilter,
}

pub trait TestCaseFilter {
    fn filter<T>(&self, test_case: &TestCase<T>) -> FilterResult
    where
        T: TestCaseIsIgnored;
}

pub trait TestCaseIsIgnored {
    fn is_ignored(&self) -> bool;
}
