use crate::scarb::config::ForgeConfigFromScarb;
use anyhow::{Context, Result, anyhow};
use camino::Utf8Path;
use configuration::Config;
use configuration::core::Profile;
use scarb_api::ScarbCommand;
use scarb_metadata::{Metadata, PackageId};
use scarb_ui::args::{FeaturesSpec, PackagesFilter, ProfileSpec};

pub mod config;

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

pub fn build_contracts_with_scarb(
    filter: PackagesFilter,
    features: FeaturesSpec,
    profile: ProfileSpec,
    manifest_path: &Utf8Path,
) -> Result<()> {
    ScarbCommand::new_with_stdio()
        .manifest_path(manifest_path)
        .arg("build")
        .packages_filter(filter)
        .features(features)
        .profile(profile)
        .run()
        .context("Failed to build contracts with Scarb")?;
    Ok(())
}

pub fn build_test_artifacts_with_scarb(
    filter: PackagesFilter,
    features: FeaturesSpec,
    profile: ProfileSpec,
    manifest_path: &Utf8Path,
) -> Result<()> {
    ScarbCommand::new_with_stdio()
        .manifest_path(manifest_path)
        .arg("build")
        .arg("--test")
        .packages_filter(filter)
        .features(features)
        .profile(profile)
        .run()
        .context("Failed to build test artifacts with Scarb")?;
    Ok(())
}
