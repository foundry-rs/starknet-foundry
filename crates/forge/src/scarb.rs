use crate::scarb::config::ForgeConfigFromScarb;
use anyhow::{Context, Result, anyhow};
use cairo_lang_sierra::program::VersionedProgram;
use camino::Utf8Path;
use configuration::Config;
use configuration::core::Profile;
use forge_runner::package_tests::TestTargetLocation;
use forge_runner::package_tests::raw::TestTargetRaw;
use scarb_api::{ScarbCommand, test_targets_by_name};
use scarb_metadata::{Metadata, PackageId, PackageMetadata};
use scarb_ui::args::{FeaturesSpec, PackagesFilter, ProfileSpec};
use semver::Version;
use std::fs;
use std::io::ErrorKind;

pub mod config;

const MINIMAL_SCARB_VERSION_TO_OPTIMIZE_COMPILATION: Version = Version::new(2, 8, 3);

impl Config for ForgeConfigFromScarb {
    fn tool_name() -> &'static str {
        "snforge"
    }

    fn from_raw(config: serde_json::Value) -> Result<Self>
    where
        Self: Sized,
    {
        serde_json::from_value(config.clone()).context("Failed to parse snforge config")
    }
}

/// Loads config for a specific package from the `Scarb.toml` file
/// # Arguments
/// * `metadata` - Scarb metadata object
/// * `package` - Id of the Scarb package
pub fn load_package_config<T: Config + Default>(
    metadata: &Metadata,
    package: &PackageId,
) -> Result<T> {
    let maybe_raw_metadata = metadata
        .get_package(package)
        .ok_or_else(|| anyhow!("Failed to find metadata for package = {package}"))?
        .tool_metadata(T::tool_name())
        .cloned();

    match maybe_raw_metadata {
        Some(raw_metadata) => configuration::core::load_config(raw_metadata, Profile::None),
        None => Ok(T::default()),
    }
}

#[must_use]
pub fn should_compile_starknet_contract_target(
    scarb_version: &Version,
    no_optimization: bool,
) -> bool {
    *scarb_version < MINIMAL_SCARB_VERSION_TO_OPTIMIZE_COMPILATION || no_optimization
}

#[tracing::instrument(skip_all, level = "debug")]
pub fn build_artifacts_with_scarb(
    filter: PackagesFilter,
    features: FeaturesSpec,
    profile: ProfileSpec,
    scarb_version: &Version,
    no_optimization: bool,
) -> Result<()> {
    if should_compile_starknet_contract_target(scarb_version, no_optimization) {
        build_contracts_with_scarb(filter.clone(), features.clone(), profile.clone())?;
    }
    build_test_artifacts_with_scarb(filter, features, profile)?;
    Ok(())
}

fn build_contracts_with_scarb(
    filter: PackagesFilter,
    features: FeaturesSpec,
    profile: ProfileSpec,
) -> Result<()> {
    ScarbCommand::new_with_stdio()
        .arg("build")
        .packages_filter(filter)
        .features(features)
        .profile(profile)
        .run()
        .context("Failed to build contracts with Scarb")?;
    Ok(())
}

fn build_test_artifacts_with_scarb(
    filter: PackagesFilter,
    features: FeaturesSpec,
    profile: ProfileSpec,
) -> Result<()> {
    ScarbCommand::new_with_stdio()
        .arg("build")
        .arg("--test")
        .packages_filter(filter)
        .features(features)
        .profile(profile)
        .run()
        .context("Failed to build test artifacts with Scarb")?;
    Ok(())
}

#[tracing::instrument(skip_all, level = "debug")]
pub fn load_test_artifacts(
    target_dir: &Utf8Path,
    package: &PackageMetadata,
) -> Result<Vec<TestTargetRaw>> {
    let mut targets = vec![];

    let dedup_targets = test_targets_by_name(package);

    for (target_name, target) in dedup_targets {
        let tests_location =
            if target.params.get("test-type").and_then(|v| v.as_str()) == Some("unit") {
                TestTargetLocation::Lib
            } else {
                TestTargetLocation::Tests
            };

        let target_file = format!("{target_name}.test.sierra.json");
        let sierra_program_path = target_dir.join(target_file);

        match fs::read_to_string(&sierra_program_path) {
            Ok(value) => {
                let versioned_program = serde_json::from_str::<VersionedProgram>(&value)?;

                let sierra_program = match versioned_program {
                    VersionedProgram::V1 { program, .. } => program,
                };

                let test_target = TestTargetRaw {
                    sierra_program,
                    sierra_program_path,
                    tests_location,
                };

                targets.push(test_target);
            }
            Err(err) if err.kind() == ErrorKind::NotFound => {}
            Err(err) => Err(err)?,
        }
    }

    Ok(targets)
}
