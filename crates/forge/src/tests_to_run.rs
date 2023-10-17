use test_collector::TestCase;

#[derive(Debug, PartialEq)]
pub(crate) enum TestsToRun {
    All,
    Ignored,
    NotIgnored,
}

impl TestsToRun {
    pub(crate) fn from_flags(only_ignored: bool, include_ignored: bool) -> Self {
        assert!(!(only_ignored && include_ignored));
        if include_ignored {
            Self::All
        } else if only_ignored {
            Self::Ignored
        } else {
            Self::NotIgnored
        }
    }
}

pub(crate) fn should_be_run(test_case: &TestCase, tests_to_run: &TestsToRun) -> bool {
    match tests_to_run {
        TestsToRun::All => true,
        TestsToRun::Ignored => test_case.ignored,
        TestsToRun::NotIgnored => !test_case.ignored,
    }
}
