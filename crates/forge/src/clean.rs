use anyhow::{Context, Result};
use scarb_api::{metadata::MetadataCommandExt, ScarbCommand};
use std::fs;
use std::path::PathBuf;

pub use super::{CleanArgs, CleanComponent};

pub fn clean(args: CleanArgs) -> Result<()> {
    // Determine the base directory for cleaning
    let scarb_metadata = ScarbCommand::metadata().inherit_stderr().run()?;
    let packages_root: Vec<PathBuf> = scarb_metadata
        .packages
        .into_iter()
        .map(|x| x.root.into())
        .collect();
    let cache_dir: PathBuf = scarb_metadata
        .workspace
        .root
        .join(".snfoundry_cache")
        .into();
    let trace_dir: PathBuf = scarb_metadata
        .workspace
        .root
        .join(".snfoundry_trace")
        .into();
    // Process each specified component
    for component in &args.clean_components {
        match component {
            CleanComponent::All => {
                clean_coverage(&packages_root)?;
                clean_profile(&packages_root)?;
                clean_cache(&cache_dir, &trace_dir)?;
            }
            CleanComponent::Coverage => clean_coverage(&packages_root)?,
            CleanComponent::Profile => clean_profile(&packages_root)?,
            CleanComponent::Cache => clean_cache(&cache_dir, &trace_dir)?,
        }
    }

    Ok(())
}

fn clean_coverage(packages_root: &Vec<PathBuf>) -> Result<()> {
    // Clean coverage directories in Scarb packages

    for package_root in packages_root {
        let coverage_dir = package_root.join("coverage");
        if coverage_dir.exists() {
            fs::remove_dir_all(&coverage_dir).with_context(|| {
                format!(
                    "Failed to remove coverage directory: {}",
                    coverage_dir.display()
                )
            })?;
            println!("Removed coverage directory: {}", coverage_dir.display());
        }
    }

    Ok(())
}

fn clean_profile(packages_root: &Vec<PathBuf>) -> Result<()> {
    // Clean profile directories in Scarb packages

    for package_root in packages_root {
        let profile_dir = package_root.join("profile");
        if profile_dir.exists() {
            fs::remove_dir_all(&profile_dir).with_context(|| {
                format!(
                    "Failed to remove profile directory: {}",
                    profile_dir.display()
                )
            })?;
            println!("Removed profile directory: {}", profile_dir.display());
        }
    }

    Ok(())
}

fn clean_cache(cache_dir: &PathBuf, trace_dir: &PathBuf) -> Result<()> {
    // Clean Scarb-specific cache and trace directories
    // let scarb_metadata = ScarbCommand::metadata().inherit_stderr().run()?;

    if cache_dir.exists() {
        fs::remove_dir_all(&cache_dir).with_context(|| {
            format!("Failed to remove cache directory: {}", cache_dir.display())
        })?;
        println!("Removed cache directory: {}", cache_dir.display());
    }

    if trace_dir.exists() {
        fs::remove_dir_all(&trace_dir).with_context(|| {
            format!("Failed to remove trace directory: {}", trace_dir.display())
        })?;
        println!("Removed trace directory: {}", trace_dir.display());
    }

    Ok(())
}
