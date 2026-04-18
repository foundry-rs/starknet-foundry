use crate::profile_validation::{check_cairo_profile_entries, get_manifest};
use anyhow::ensure;
use indoc::formatdoc;
use scarb_metadata::Metadata;

/// Checks if debugger can be launched based on profile settings extracted from the provided [`Metadata`].
pub fn check_debugger_compatibility(scarb_metadata: &Metadata) -> anyhow::Result<()> {
    const DEBUGGER_REQUIRED_ENTRIES: &[(&str, &str)] = &[
        ("unstable-add-statements-code-locations-debug-info", "true"),
        ("unstable-add-statements-functions-debug-info", "true"),
        ("add-functions-debug-info", "true"),
        ("skip-optimizations", "true"),
    ];

    let manifest = get_manifest(scarb_metadata)?;

    let has_needed_entries =
        check_cairo_profile_entries(&manifest, scarb_metadata, DEBUGGER_REQUIRED_ENTRIES);

    ensure!(
        has_needed_entries,
        formatdoc! {
            "Scarb.toml must have the following Cairo compiler configuration to launch the debugger:

            [profile.{profile}.cairo]
            unstable-add-statements-code-locations-debug-info = true
            unstable-add-statements-functions-debug-info = true
            add-functions-debug-info = true
            skip-optimizations = true
            ... other entries ...
            ",
            profile = scarb_metadata.current_profile
        },
    );

    Ok(())
}
