use crate::collecting::CompiledTestCrateRaw;
use anyhow::{anyhow, Context, Result};
use camino::Utf8Path;
use scarb_ui::args::PackagesFilter;
use std::process::{Command, Stdio};

pub fn load_test_artifacts(
    snforge_target_dir_path: &Utf8Path,
    package_name: &str,
) -> Result<Vec<CompiledTestCrateRaw>> {
    let snforge_test_artifact_path =
        snforge_target_dir_path.join(format!("{package_name}.snforge_sierra.json"));
    let test_crates = serde_json::from_str::<Vec<CompiledTestCrateRaw>>(&std::fs::read_to_string(
        snforge_test_artifact_path,
    )?)?;
    Ok(test_crates)
}

pub fn build_test_artifacts_with_scarb(filter: PackagesFilter) -> Result<()> {
    let build_output = Command::new("scarb")
        .arg("snforge-test-collector")
        .env("SCARB_PACKAGES_FILTER", filter.to_env())
        .stderr(Stdio::inherit())
        .stdout(Stdio::inherit())
        .output()
        .context("Failed to build test artifacts with Scarb")?;

    if build_output.status.success() {
        Ok(())
    } else {
        Err(anyhow!("scarb snforge-test-collector did not succeed"))
    }
}
