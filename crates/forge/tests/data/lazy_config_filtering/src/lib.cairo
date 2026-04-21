#[cfg(test)]
mod tests {
    #[test]
    fn selected_exact() {
        assert(true, 'ok');
    }

    // This config is intentionally invalid.
    // When runninng with `--exact lazy_config_filtering::tests::selected_exact`, this test
    // should be filtered out and thus not fail.
    #[test]
    #[fork("missing_fork")]
    fn filtered_out_broken_config() {
        assert(true, 'should never run');
    }
}
