/// Asserts a cleaned `stdout` snapshot using `insta`, filtered for non-deterministic lines.
/// Uses the current Scarb version as a snapshot suffix.
#[macro_export]
macro_rules! assert_cleaned_output {
    ($output:expr) => {{
        let stdout = String::from_utf8_lossy(&$output.get_output().stdout);

        let scarb_version = scarb_api::ScarbCommand::version()
            .run()
            .expect("Failed to get scarb version")
            .scarb;

        insta::with_settings!({
            snapshot_suffix => scarb_version.to_string(),
            filters => vec![
                (r"(?m)^\s*(Compiling|Finished|Blocking).*", ""), // scarb output
                (r"(?m)^\s*(Collected|Running|Tests:|Latest block number).*", ""), // snforge output
            ]},
            {
                insta::assert_snapshot!(stdout);
            }
        );
    }};
}
