use anyhow::{Context, Result};
use scarb_artifacts::{get_contracts_map, target_dir_for_package, StarknetContractArtifacts};
use scarb_metadata::{Metadata, PackageMetadata};
use scarb_ui::args::PackagesFilter;
use std::collections::HashMap;
use std::fs;
use std::process::{Command, Stdio};

pub const LIB_CONTRACT_ARTIFACTS_NAME: &str = "__sncast_lib_contract";

#[allow(clippy::too_many_lines)]
pub fn build(
    metadata: &Metadata,
    package: &PackageMetadata,
    for_scripts: bool,
) -> Result<HashMap<String, StarknetContractArtifacts>> {
    let filter = PackagesFilter::generate_for::<Metadata>([package].into_iter());

    let command_result = Command::new("scarb")
        .env("SCARB_PACKAGES_FILTER", filter.to_env())
        .arg("--manifest-path")
        .arg(&package.manifest_path)
        .arg("build")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .context("Failed to start building contracts with Scarb")?;

    let result_code = command_result
        .status
        .code()
        .context("Failed to obtain status code from scarb build")?;
    let result_msg = String::from_utf8(command_result.stdout)?;
    if result_code != 0 {
        anyhow::bail!(
            "Scarb build returned non-zero exit code: {} - error message: {}",
            result_code,
            result_msg
        );
    }

    let mut contracts = get_contracts_map(metadata, &package.id)?;

    if for_scripts {
        // Insert lib "contract"
        let sierra_filename = format!("{}.sierra.json", package.name);

        let sierra_path = target_dir_for_package(metadata)
            .join(&metadata.current_profile)
            .join(sierra_filename);

        let lib_artifacts = StarknetContractArtifacts {
            sierra: fs::read_to_string(sierra_path)?,
            casm: String::new(), // There seems to be no need for casm in lib
        };

        contracts.insert(LIB_CONTRACT_ARTIFACTS_NAME.to_owned(), lib_artifacts);
    }

    Ok(contracts)
}
