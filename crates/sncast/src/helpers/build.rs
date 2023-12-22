use anyhow::{Context, Result};
use camino::Utf8PathBuf;
use scarb_artifacts::{
    get_contracts_map, target_dir_for_package, ScarbCommand, StarknetContractArtifacts,
};
use scarb_metadata::{Metadata, PackageMetadata};
use scarb_ui::args::PackagesFilter;
use std::collections::HashMap;
use std::fs;

use super::constants::LIB_CONTRACT_ARTIFACTS_NAME;

pub struct BuildConfig {
    pub scarb_toml_path: Option<Utf8PathBuf>,
    pub json: bool,
}

#[allow(clippy::too_many_lines)]
pub fn build(
    metadata: &Metadata,
    package: &PackageMetadata,
    for_scripts: bool,
    build_config: &BuildConfig,
) -> Result<HashMap<String, StarknetContractArtifacts>> {
    let filter = PackagesFilter::generate_for::<Metadata>([package].into_iter());

    let mut command = ScarbCommand::new_with_stdio();
    command
        .arg("build")
        .manifest_path(&package.manifest_path)
        .packages_filter(filter);
    if build_config.json {
        command.json();
    }
    command
        .run()
        .context("Failed to build contracts with Scarb")?;

    let mut artifacts = get_contracts_map(metadata, &package.id)?;

    if for_scripts {
        let sierra_filename = format!("{}.sierra.json", package.name);

        let sierra_path = target_dir_for_package(metadata)
            .join(&metadata.current_profile)
            .join(sierra_filename);

        let lib_artifacts = StarknetContractArtifacts {
            sierra: fs::read_to_string(sierra_path)?,
            casm: String::new(), // There seems to be no need for casm in lib
        };

        artifacts.insert(LIB_CONTRACT_ARTIFACTS_NAME.to_owned(), lib_artifacts);
    }

    Ok(artifacts)
}
