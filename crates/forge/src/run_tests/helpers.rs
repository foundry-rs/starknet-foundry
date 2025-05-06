pub fn skip_fork_tests() -> bool {
    std::env::var("SNFORGE_SKIP_FORK_TESTS")
        .map(|v| v == "1")
        .unwrap_or(false)
}
