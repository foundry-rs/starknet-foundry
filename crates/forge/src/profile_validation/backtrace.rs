use crate::TestArgs;
use crate::profile_validation::{bool_field, bool_field_or_unstable};
use anyhow::ensure;
use camino::Utf8PathBuf;
use indoc::formatdoc;
use serde_json::Value;

#[allow(unused_variables)]
pub fn check_backtrace_compatibility(
    test_args: &TestArgs,
    compiler_config: &Value,
    profile: &str,
    workspace_manifest_path: &Utf8PathBuf,
) -> anyhow::Result<()> {
    #[cfg(feature = "cairo-native")]
    check_if_native_disabled(test_args)?;
    check_profile(compiler_config, profile, workspace_manifest_path)
}

/// Checks if native execution is disabled in the provided [`TestArgs`].
#[cfg(feature = "cairo-native")]
fn check_if_native_disabled(test_args: &TestArgs) -> anyhow::Result<()> {
    ensure!(
        !test_args.run_native,
        "Backtrace generation is not supported with `cairo-native` execution",
    );
    Ok(())
}

fn check_profile(
    compiler_config: &Value,
    profile: &str,
    workspace_manifest_path: &Utf8PathBuf,
) -> anyhow::Result<()> {
    let has_needed_entries =
        bool_field_or_unstable(compiler_config, "add_statements_code_locations_debug_info")
            && bool_field_or_unstable(compiler_config, "add_statements_functions_debug_info")
            && bool_field(compiler_config, "panic_backtrace");

    ensure!(
        has_needed_entries,
        formatdoc! {
            "{workspace_manifest_path} must have the following Cairo compiler configuration to run backtrace:

            [profile.{profile}.cairo]
            unstable-add-statements-functions-debug-info = true
            unstable-add-statements-code-locations-debug-info = true
            panic-backtrace = true
            ... other entries ...
            ",
        },
    );

    Ok(())
}
