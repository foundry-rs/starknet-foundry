use crate::{CleanArgs, CleanComponent};
use anyhow::{ensure, Context, Result};
use camino::Utf8PathBuf;
use scarb_api::{metadata::MetadataCommandExt, ScarbCommand};
use std::fs;

const COVERAGE_DIR: &str = "coverage";
const PROFILE_DIR: &str = "profile";
const CACHE_DIR: &str = ".snfoundry_cache";
const TRACE_DIR: &str = "snfoundry_trace";

pub fn clean(args: CleanArgs) -> Result<()> {
    let components = if args.clean_components.contains(&CleanComponent::All) {
        ensure!(
            args.clean_components.len() == 1,
            "The 'all' component cannot be combined with other components"
        );
        vec![
            CleanComponent::Trace,
            CleanComponent::Profile,
            CleanComponent::Cache,
            CleanComponent::Coverage,
        ]
    } else {
        args.clean_components
    };

    let scarb_metadata = ScarbCommand::metadata().inherit_stderr().no_deps().run()?;
    let workspace_root = scarb_metadata.workspace.root;

    let packages_root: Vec<Utf8PathBuf> = scarb_metadata
        .packages
        .into_iter()
        .map(|package_metada| package_metada.root)
        .collect();

    for component in &components {
        match component {
            CleanComponent::Coverage => clean_dirs(&packages_root, COVERAGE_DIR)?,
            CleanComponent::Profile => clean_dirs(&packages_root, PROFILE_DIR)?,
            CleanComponent::Cache => clean_dir(&workspace_root, CACHE_DIR)?,
            CleanComponent::Trace => clean_dir(&workspace_root, TRACE_DIR)?,
            CleanComponent::All => unreachable!("All component should have been handled earlier"),
        }
    }

    Ok(())
}

fn clean_dirs(root_dirs: &[Utf8PathBuf], dir_name: &str) -> Result<()> {
    for root_dir in root_dirs {
        clean_dir(root_dir, dir_name)?;
    }
    Ok(())
}
fn clean_dir(dir: &Utf8PathBuf, dir_name: &str) -> Result<()> {
    let dir = dir.join(dir_name);
    if dir.exists() {
        fs::remove_dir_all(&dir).with_context(|| format!("Failed to remove directory: {dir}"))?;
        println!("Removed directory: {dir}");
    }

    Ok(())
}
