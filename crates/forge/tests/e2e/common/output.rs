/// Asserts a cleaned `stdout` snapshot using `insta`, filtered for non-deterministic lines.
/// Uses the current Scarb version as a snapshot suffix.
#[macro_export]
macro_rules! assert_cleaned_output {
    ($output:expr) => {{
        let stdout = String::from_utf8_lossy(&$output.get_output().stdout);

        let scarb_version = scarb_api::version::scarb_version()
            .expect("Failed to get scarb version")
            .scarb;

        // Extract the module name from module_path to determine snapshot subdirectory
        let module_path = module_path!();
        let snapshot_subdir = module_path
            .split("::")
            .last() // Get the last module name (e.g., "backtrace" or "gas_report")
            .unwrap_or("");

        insta::with_settings!({
            snapshot_suffix => scarb_version.to_string(),
            snapshot_path => format!("./snapshots/{}", snapshot_subdir),
            filters => vec![
                (r"\x1B\[[0-?]*[ -/]*[@-~]", ""), // ANSI escape regex - needed for CI
                (r"(?m)^\s*(Compiling|Finished|Blocking).*", ""), // scarb output
                (r"(?m)^\s*(Collected|Running|Tests:|Latest block number).*", ""), // snforge output
                (r"(?m)^\s*(Updating crates\.io index|warning:.*|This may prevent.*|database is locked.*|Caused by:.*|  Error code.*).*\n", ""), // cargo warnings and errors
                (r"(?m)^\s*(Downloading crates|Downloaded).*\n", ""), // cargo download output
                (r"at /[^\s:]+/src/", "at [..]"), // absolute paths in backtrace
            ]},
            {
                insta::assert_snapshot!(stdout);
            }
        );
    }};
}
