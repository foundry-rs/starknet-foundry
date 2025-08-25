use crate::profile_validation::{check_cairo_profile_entries, get_manifest};
use anyhow::ensure;
use indoc::formatdoc;
use scarb_metadata::Metadata;
use semver::Version;

/// Checks if coverage can be based on scarb version and profile settings extracted from the provided [`Metadata`].
pub fn check_coverage_compatibility(scarb_metadata: &Metadata) -> anyhow::Result<()> {
    check_scarb_version(scarb_metadata)?;
    check_profile(scarb_metadata)?;
    Ok(())
}

/// Checks if the scarb version from the provided [`Metadata`] is greater than or equal to the minimal required version.
fn check_scarb_version(scarb_metadata: &Metadata) -> anyhow::Result<()> {
    const MINIMAL_SCARB_VERSION: Version = Version::new(2, 8, 0);
    ensure!(
        scarb_metadata.app_version_info.version >= MINIMAL_SCARB_VERSION,
        "Coverage generation requires scarb version >= {MINIMAL_SCARB_VERSION}",
    );
    Ok(())
}

/// Checks if the runtime profile settings in the provided from [`Metadata`] contain the required entries for coverage generation.
fn check_profile(scarb_metadata: &Metadata) -> anyhow::Result<()> {
    const CAIRO_COVERAGE_REQUIRED_ENTRIES: &[(&str, &str)] = &[
        ("unstable-add-statements-functions-debug-info", "true"),
        ("unstable-add-statements-code-locations-debug-info", "true"),
        ("inlining-strategy", "avoid"),
    ];

    let manifest = get_manifest(scarb_metadata)?;

    let has_needed_entries =
        check_cairo_profile_entries(&manifest, scarb_metadata, CAIRO_COVERAGE_REQUIRED_ENTRIES);

    ensure!(
        has_needed_entries,
        formatdoc! {
            "Scarb.toml must have the following Cairo compiler configuration to run coverage:

            [profile.{profile}.cairo]
            unstable-add-statements-functions-debug-info = true
            unstable-add-statements-code-locations-debug-info = true
            inlining-strategy = \"avoid\"
            ... other entries ...
            ",
            profile = scarb_metadata.current_profile
        },
    );

    Ok(())
}
