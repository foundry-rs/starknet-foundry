use crate::collecting::CompiledTestCrate;
use test_collector::TestCase;

#[derive(Debug, PartialEq)]
// Specifies what tests should be included
pub(crate) struct TestsFilter {
    // based on name
    pub name_filter: NameFilter,
    // based on `#[ignore]` attribute
    pub ignored_filter: IgnoredFilter,
}

#[derive(Debug, PartialEq)]
pub(crate) enum NameFilter {
    All,
    Match(String),
    ExactMatch(String),
}

#[derive(Debug, PartialEq)]
pub(crate) enum IgnoredFilter {
    NotIgnored,
    Ignored,
    All,
}

impl TestsFilter {
    pub(crate) fn new(
        test_name_filter: Option<String>,
        exact_match: bool,
        only_ignored: bool,
        include_ignored: bool,
    ) -> Self {
        assert!(!(only_ignored && include_ignored));

        let ignored_filter = if include_ignored {
            IgnoredFilter::All
        } else if only_ignored {
            IgnoredFilter::Ignored
        } else {
            IgnoredFilter::NotIgnored
        };

        let name_filter = if exact_match {
            NameFilter::ExactMatch(test_name_filter.unwrap())
        } else if let Some(name) = test_name_filter {
            NameFilter::Match(name)
        } else {
            NameFilter::All
        };

        Self {
            name_filter,
            ignored_filter,
        }
    }

    pub(crate) fn filter_tests(&self, test_crate: CompiledTestCrate) -> CompiledTestCrate {
        let mut cases = test_crate.test_cases;

        cases = match &self.name_filter {
            NameFilter::All => cases,
            NameFilter::Match(filter) => cases
                .into_iter()
                .filter(|tc| tc.name.contains(filter))
                .collect(),
            NameFilter::ExactMatch(name) => {
                cases.into_iter().filter(|tc| tc.name == *name).collect()
            }
        };

        cases = match self.ignored_filter {
            // if NotIgnored (default) we filter ignored tests later and display them as ignored
            IgnoredFilter::All | IgnoredFilter::NotIgnored => cases,
            IgnoredFilter::Ignored => cases.into_iter().filter(|tc| tc.ignored).collect(),
        };

        CompiledTestCrate {
            test_cases: cases,
            ..test_crate
        }
    }

    pub(crate) fn should_be_run(&self, test_case: &TestCase) -> bool {
        match self.ignored_filter {
            IgnoredFilter::All => true,
            IgnoredFilter::Ignored => test_case.ignored,
            IgnoredFilter::NotIgnored => !test_case.ignored,
        }
    }
}
