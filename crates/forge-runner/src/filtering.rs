use crate::package_tests::TestCase;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NameFilter {
    All,
    Match(String),
    ExactMatch(String),
}

impl NameFilter {
    #[must_use]
    pub fn from_flags(test_name_filter: Option<String>, exact_match: bool) -> Self {
        if exact_match {
            Self::ExactMatch(
                test_name_filter
                    .expect("Argument test_name_filter cannot be None with exact_match"),
            )
        } else if let Some(name) = test_name_filter {
            Self::Match(name)
        } else {
            Self::All
        }
    }

    #[must_use]
    pub fn matches(&self, sanitized_name: &str) -> bool {
        match self {
            Self::All => true,
            Self::Match(filter) => sanitized_name.contains(filter),
            Self::ExactMatch(name) => sanitized_name == name,
        }
    }

    #[must_use]
    pub fn exact_match(&self) -> Option<&str> {
        match self {
            Self::ExactMatch(name) => Some(name),
            Self::All | Self::Match(_) => None,
        }
    }
}

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
    /// Test case is ignored.
    Ignored,
    /// Test case is excluded from the current partition.
    ExcludedFromPartition,
}

pub trait TestCaseFilter {
    fn filter<T>(&self, test_case: &TestCase<T>) -> FilterResult
    where
        T: TestCaseIsIgnored;
}

pub trait TestCaseIsIgnored {
    fn is_ignored(&self) -> bool;
}

#[cfg(test)]
mod tests {
    use super::NameFilter;

    #[test]
    fn name_filter_all_matches_everything() {
        assert!(NameFilter::All.matches("any::test"));
    }

    #[test]
    fn name_filter_match_uses_substring_matching() {
        assert!(NameFilter::Match("selected".to_string()).matches("pkg::selected_test"));
        assert!(!NameFilter::Match("other".to_string()).matches("pkg::selected_test"));
    }

    #[test]
    fn name_filter_exact_match_uses_equality() {
        assert!(NameFilter::ExactMatch("pkg::test".to_string()).matches("pkg::test"));
        assert!(!NameFilter::ExactMatch("pkg::test".to_string()).matches("pkg::test_case"));
    }

    #[test]
    fn name_filter_exact_match_helper_returns_only_exact_filter() {
        assert_eq!(
            NameFilter::ExactMatch("pkg::test".to_string()).exact_match(),
            Some("pkg::test")
        );
        assert_eq!(NameFilter::All.exact_match(), None);
        assert_eq!(NameFilter::Match("pkg".to_string()).exact_match(), None);
    }
}
