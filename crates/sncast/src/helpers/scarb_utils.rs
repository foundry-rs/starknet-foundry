use crate::helpers::artifacts::CastStarknetContractArtifacts;
use anyhow::{Context, Result, anyhow};
use camino::{Utf8Path, Utf8PathBuf};
use foundry_ui::{UI, components::warning::WarningMessage};
use scarb_api::metadata::{MetadataError, MetadataOpts, metadata_for_dir, metadata_with_opts};
use scarb_api::{
    CompilationOpts, ScarbCommand, ScarbCommandError, ensure_scarb_available,
    get_contracts_artifacts_and_source_sierra_paths,
    metadata::{Metadata, PackageMetadata},
    target_dir_for_workspace,
};
use scarb_ui::args::PackagesFilter;
use shared::command::CommandExt;
use std::collections::HashMap;
use std::str::FromStr;

pub fn get_scarb_manifest() -> Result<Utf8PathBuf> {
    get_scarb_manifest_for(<&Utf8Path>::from("."))
}

pub fn get_scarb_manifest_for(dir: &Utf8Path) -> Result<Utf8PathBuf> {
    ensure_scarb_available()?;

    let output = ScarbCommand::new()
        .current_dir(dir)
        .arg("manifest-path")
        .command()
        .output_checked()
        .context("Failed to execute the `scarb manifest-path` command")?;

    let output_str = String::from_utf8(output.stdout)
        .context("`scarb manifest-path` command failed to provide valid output")?;

    let path = Utf8PathBuf::from_str(output_str.trim())
        .context("`scarb manifest-path` failed. Invalid location returned")?;

    Ok(path)
}

pub fn get_scarb_metadata(manifest_path: &Utf8PathBuf) -> Result<Metadata, MetadataError> {
    metadata_with_opts(MetadataOpts {
        current_dir: Some(
            manifest_path
                .parent()
                .expect("manifest should have parent")
                .into(),
        ),
        no_deps: true,
        ..MetadataOpts::default()
    })
}

pub fn get_scarb_metadata_with_deps(
    manifest_path: &Utf8PathBuf,
) -> Result<Metadata, MetadataError> {
    metadata_for_dir(manifest_path.parent().expect("manifest should have parent"))
}

pub fn get_cairo_version(manifest_path: &Utf8PathBuf) -> Result<String> {
    let scarb_metadata = get_scarb_metadata(manifest_path)?;

    Ok(scarb_metadata.app_version_info.cairo.version.to_string())
}

pub fn assert_manifest_path_exists() -> Result<Utf8PathBuf> {
    let manifest_path = get_scarb_manifest().expect("Failed to obtain manifest path from scarb");

    if !manifest_path.exists() {
        return Err(anyhow!(
            "Path to Scarb.toml manifest does not exist = {manifest_path}"
        ));
    }

    Ok(manifest_path)
}

fn get_package_metadata_by_name<'a>(
    metadata: &'a Metadata,
    package_name: &str,
) -> Result<&'a PackageMetadata> {
    metadata
        .packages
        .iter()
        .find(|package| package.name == package_name)
        .ok_or(anyhow!(
            "Package {} not found in scarb metadata",
            &package_name
        ))
}

fn get_default_package_metadata(metadata: &Metadata) -> Result<&PackageMetadata> {
    match metadata.packages.iter().collect::<Vec<_>>().as_slice() {
        [package] => Ok(package),
        [] => Err(anyhow!("No package found in scarb metadata")),
        _ => Err(anyhow!(
            "More than one package found in scarb metadata - specify package using --package flag"
        )),
    }
}

pub fn get_package_metadata(
    manifest_path: &Utf8PathBuf,
    package_name: &Option<String>,
) -> Result<PackageMetadata> {
    let metadata = get_scarb_metadata(manifest_path)?;
    match &package_name {
        Some(package_name) => Ok(get_package_metadata_by_name(&metadata, package_name)?.clone()),
        None => Ok(get_default_package_metadata(&metadata)?.clone()),
    }
}

pub struct BuildConfig {
    pub scarb_toml_path: Utf8PathBuf,
    pub json: bool,
    pub profile: String,
}

pub fn build(
    package: &PackageMetadata,
    config: &BuildConfig,
    default_profile: &str,
) -> Result<(), ScarbCommandError> {
    let filter = PackagesFilter::generate_for::<Metadata>([package].into_iter());

    let mut cmd = ScarbCommand::new_with_stdio();
    let metadata =
        get_scarb_metadata_with_deps(&config.scarb_toml_path).expect("Failed to obtain metadata");
    let profile = if metadata.profiles.contains(&config.profile) {
        &config.profile
    } else {
        default_profile
    };
    cmd.arg("--profile")
        .arg(profile)
        .arg("build")
        .manifest_path(&config.scarb_toml_path)
        .packages_filter(filter);

    if config.json {
        cmd.json();
    }
    cmd.run()
}

pub fn build_and_load_artifacts(
    package: &PackageMetadata,
    config: &BuildConfig,
    build_for_script: bool,
    ui: &UI,
) -> Result<HashMap<String, CastStarknetContractArtifacts>> {
    // TODO (#2042): Remove this logic, always use release as default
    let default_profile = if build_for_script { "dev" } else { "release" };
    build(package, config, default_profile)
        .map_err(|e| anyhow!(format!("Failed to build using scarb; {e}")))?;

    let metadata = get_scarb_metadata_with_deps(&config.scarb_toml_path)?;
    let target_dir = target_dir_for_workspace(&metadata);

    if metadata.profiles.contains(&config.profile) {
        Ok(get_contracts_artifacts_and_source_sierra_paths(
            &target_dir.join(&config.profile),
            package,
            ui,
            CompilationOpts::default()
        ).context("Failed to load artifacts. Make sure you have enabled sierra code generation in Scarb.toml")?
        .into_iter()
        .map(|(name, (artifacts, _))| (name, CastStarknetContractArtifacts { sierra: artifacts.sierra, casm: serde_json::to_string(&artifacts.casm).expect("valid serialization")  }))
        .collect())
    } else {
        let profile = &config.profile;
        ui.println(&WarningMessage::new(&format!(
            "Profile {profile} does not exist in scarb, using '{default_profile}' profile."
        )));
        Ok(get_contracts_artifacts_and_source_sierra_paths(
            &target_dir.join(default_profile),
            package,
            ui,
            CompilationOpts::default(),
        ).context("Failed to load artifacts. Make sure you have enabled sierra code generation in Scarb.toml")?
        .into_iter()
        .map(|(name, (artifacts, _))| (name, CastStarknetContractArtifacts { sierra: artifacts.sierra, casm: serde_json::to_string(&artifacts.casm).expect("valid serialization") }))
        .collect())
    }
}

#[cfg(test)]
mod tests {
    use crate::helpers::scarb_utils::{get_package_metadata, get_scarb_metadata};

    #[test]
    fn test_get_scarb_metadata() {
        let metadata =
            get_scarb_metadata(&"tests/data/contracts/constructor_with_params/Scarb.toml".into());
        assert!(metadata.is_ok());
    }

    #[test]
    fn test_get_package_metadata_happy_default() {
        let metadata = get_package_metadata(
            &"tests/data/contracts/constructor_with_params/Scarb.toml".into(),
            &None,
        )
        .unwrap();
        assert_eq!(metadata.name, "constructor_with_params");
    }

    #[test]
    fn test_get_package_metadata_happy_by_name() {
        let metadata = get_package_metadata(
            &"tests/data/contracts/multiple_packages/Scarb.toml".into(),
            &Some("package2".into()),
        )
        .unwrap();
        assert_eq!(metadata.name, "package2");
    }

    #[test]
    #[should_panic(
        expected = "More than one package found in scarb metadata - specify package using --package flag"
    )]
    fn test_get_package_metadata_more_than_one_default() {
        get_package_metadata(
            &"tests/data/contracts/multiple_packages/Scarb.toml".into(),
            &None,
        )
        .unwrap();
    }

    #[test]
    #[should_panic(expected = "Package whatever not found in scarb metadata")]
    fn test_get_package_metadata_no_such_package() {
        let metadata = get_package_metadata(
            &"tests/data/contracts/multiple_packages/Scarb.toml".into(),
            &Some("whatever".into()),
        )
        .unwrap();
        assert_eq!(metadata.name, "package2");
    }
}
