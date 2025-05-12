#[must_use]
pub fn env_ignore_fork_tests() -> bool {
    std::env::var("SNFORGE_IGNORE_FORK_TESTS").is_ok_and(|v| v == "1")
}
