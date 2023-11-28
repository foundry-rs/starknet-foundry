use anyhow::{Context, Result};
use scarb_artifacts::{get_contracts_map, StarknetContractArtifacts};
use scarb_metadata::{Metadata, PackageMetadata};
use scarb_ui::args::PackagesFilter;
use std::collections::HashMap;
use std::process::{Command, Stdio};

#[allow(clippy::too_many_lines)]
pub fn build(
    metadata: &Metadata,
    package: &PackageMetadata,
) -> Result<HashMap<String, StarknetContractArtifacts>> {
    let filter = PackagesFilter::generate_for::<Metadata>([package].into_iter());

    let command_result = Command::new("scarb")
        .env("SCARB_PACKAGES_FILTER", filter.to_env())
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

    get_contracts_map(metadata, &package.id)
}
