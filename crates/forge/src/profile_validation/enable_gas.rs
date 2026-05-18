use crate::profile_validation::bool_field_with_default;
use anyhow::ensure;
use indoc::formatdoc;
use scarb_api::test_targets_by_name;
use scarb_metadata::{Metadata, PackageMetadata};

/// Validates test targets are compiled with enabled gas.
pub fn check_enable_gas(
    scarb_metadata: &Metadata,
    packages_to_build: &[PackageMetadata],
) -> anyhow::Result<()> {
    let workspace_manifest_path = &scarb_metadata.workspace.manifest_path;
    let profile = &scarb_metadata.current_profile;

    for package in packages_to_build {
        for test_target in test_targets_by_name(package).values() {
            let enable_gas = scarb_metadata
                .compilation_units
                .iter()
                .find(|cu| &cu.target == *test_target)
                .is_none_or(|cu| bool_field_with_default(&cu.compiler_config, "enable_gas", true));

            ensure!(
                enable_gas,
                formatdoc! {
                    "`snforge test` does not support gas calculation being disabled
                    help: enable gas calculation by adding the following entry to {workspace_manifest_path}:

                    [profile.{profile}.cairo]
                    enable-gas = true
                    ... other entries ...
                    ",
                },
            );
        }
    }

    Ok(())
}
