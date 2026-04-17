use crate::profile_validation::{bool_field_or_unstable, str_field};
use anyhow::ensure;
use camino::Utf8PathBuf;
use indoc::formatdoc;
use serde_json::Value;

pub fn check_coverage_compatibility(
    compiler_config: &Value,
    profile: &str,
    workspace_manifest_path: &Utf8PathBuf,
) -> anyhow::Result<()> {
    let has_needed_entries =
        bool_field_or_unstable(compiler_config, "add_statements_code_locations_debug_info")
            && bool_field_or_unstable(compiler_config, "add_statements_functions_debug_info")
            && (compiler_config
                .get("compiler_optimizations")
                .and_then(|v| v.get("Enabled"))
                .and_then(|v| v.get("inlining_strategy"))
                .and_then(|v| v.as_str())
                == Some("avoid")
                // When optimizations are disabled, the inlining strategy is set to `avoid`.
                || str_field(compiler_config, "compiler_optimizations") == "Disabled"
                // For compatibility with older Scarb versions.
                || str_field(compiler_config, "inlining_strategy") == "avoid");

    ensure!(
        has_needed_entries,
        formatdoc! {
            "{workspace_manifest_path} must have the following Cairo compiler configuration to run coverage:

            [profile.{profile}.cairo]
            unstable-add-statements-functions-debug-info = true
            unstable-add-statements-code-locations-debug-info = true
            inlining-strategy = \"avoid\"
            ... other entries ...
            ",
        },
    );

    Ok(())
}
