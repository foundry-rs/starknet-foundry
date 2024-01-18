use crate::collecting::CompiledTestCrateRaw;
use anyhow::{Context, Result};
use camino::Utf8Path;
use scarb_api::ScarbCommand;
use scarb_ui::args::PackagesFilter;

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
    ScarbCommand::new_with_stdio()
        .arg("snforge-test-collector")
        .packages_filter(filter)
        .run()
        .context("scarb snforge-test-collector did not succeed")?;
    Ok(())
}
