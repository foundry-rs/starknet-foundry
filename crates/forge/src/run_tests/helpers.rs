use forge_runner::package_tests::with_config::TestCaseConfig;

#[must_use]
pub fn skip_fork_tests() -> bool {
    std::env::var("SNFORGE_SKIP_FORK_TESTS")
        .map(|v| v == "1")
        .unwrap_or(false)
}

#[must_use]
pub fn is_test_case_ignored(case_config: &TestCaseConfig, skip_fork_tests_from_env: bool) -> bool {
    case_config.ignored || (skip_fork_tests_from_env && case_config.fork_config.is_some())
}
