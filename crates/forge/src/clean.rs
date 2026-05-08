use crate::{CleanArgs, CleanComponent};
use anyhow::{Context, Result, ensure};
use camino::{Utf8Path, Utf8PathBuf};
use forge_runner::resolve_cache_dir;
use foundry_ui::UI;
use scarb_api::metadata::{MetadataOpts, metadata_with_opts};
use std::fs;

const COVERAGE_DIR: &str = "coverage";
const PROFILE_DIR: &str = "profile";
const TRACE_DIR: &str = "snfoundry_trace";

pub fn clean(args: CleanArgs, ui: &UI) -> Result<()> {
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

    let scarb_metadata = metadata_with_opts(MetadataOpts {
        no_deps: true,
        ..MetadataOpts::default()
    })?;
    let workspace_root = scarb_metadata.workspace.root;

    let packages_root: Vec<Utf8PathBuf> = scarb_metadata
        .packages
        .into_iter()
        .map(|package_metadata| package_metadata.root)
        .collect();

    for component in &components {
        match component {
            CleanComponent::Coverage => packages_root
                .iter()
                .try_for_each(|root| clean_dir(&root.join(COVERAGE_DIR), ui))?,
            CleanComponent::Profile => packages_root
                .iter()
                .try_for_each(|root| clean_dir(&root.join(PROFILE_DIR), ui))?,
            CleanComponent::Cache => clean_dir(&resolve_cache_dir(&workspace_root), ui)?,
            CleanComponent::Trace => clean_dir(&workspace_root.join(TRACE_DIR), ui)?,
            CleanComponent::All => unreachable!("All component should have been handled earlier"),
        }
    }

    Ok(())
}

fn clean_dir(path: &Utf8Path, ui: &UI) -> Result<()> {
    if path.exists() {
        fs::remove_dir_all(path).with_context(|| format!("Failed to remove directory: {path}"))?;
        ui.println(&format!("Removed directory: {path}"));
    }

    Ok(())
}
