use crate::profile_validation::{bool_field, bool_field_or_unstable, str_field};
use anyhow::ensure;
use camino::Utf8PathBuf;
use indoc::formatdoc;
use serde_json::Value;

pub fn check_debugger_compatibility(
    compiler_config: &Value,
    profile: &str,
    workspace_manifest_path: &Utf8PathBuf,
) -> anyhow::Result<()> {
    let has_needed_entries = bool_field(compiler_config, "add_functions_debug_info")
        && bool_field_or_unstable(compiler_config, "add_statements_code_locations_debug_info")
        && bool_field_or_unstable(compiler_config, "add_statements_functions_debug_info")
        && str_field(compiler_config, "compiler_optimizations") == "Disabled";

    ensure!(
        has_needed_entries,
        formatdoc! {
            "{workspace_manifest_path} must have the Cairo compiler configuration equivalent to the following one to launch the debugger:

            [profile.{profile}.cairo]
            unstable-add-statements-code-locations-debug-info = true
            unstable-add-statements-functions-debug-info = true
            add-functions-debug-info = true
            skip-optimizations = true
            ... other entries ...
            ",
        },
    );

    Ok(())
}
