use crate::{CleanArgs, CleanComponent};
use anyhow::{Context, Result, ensure};
use camino::{Utf8Path, Utf8PathBuf};
use forge_runner::resolve_cache_dir;
use foundry_ui::UI;
use scarb_api::metadata::{MetadataOpts, metadata_with_opts};
use std::env;
use std::fs;

const COVERAGE_DIR: &str = "coverage";
const FILE_WITH_PREV_TESTS_FAILED: &str = ".prev_tests_failed";
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
            CleanComponent::Cache => clean_cache_dir(&resolve_cache_dir(&workspace_root)?, ui)?,
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

pub fn clean_cache_dir(path: &Utf8Path, ui: &UI) -> Result<()> {
    if !path.exists() {
        return Ok(());
    }

    let has_custom_cache_dir = env::var("SNFOUNDRY_CACHE").is_ok();
    if !has_custom_cache_dir {
        return clean_dir(path, ui);
    }

    let mut any_skipped = false;
    for entry in path
        .read_dir_utf8()
        .with_context(|| format!("Failed to read cache directory: {path}"))?
    {
        let entry = entry.with_context(|| format!("Failed to read cache directory: {path}"))?;
        let entry_path = entry.path();

        if is_snfoundry_cache_file(entry_path) {
            fs::remove_file(entry_path)
                .with_context(|| format!("Failed to remove cache file: {entry_path}"))?;
            ui.println(&format!("Removed file: {entry_path}"));
        } else {
            any_skipped = true;
        }
    }

    if !any_skipped {
        fs::remove_dir(path).with_context(|| format!("Failed to remove directory: {path}"))?;
        ui.println(&format!("Removed directory: {path}"));
    }

    Ok(())
}

fn is_snfoundry_cache_file(path: &Utf8Path) -> bool {
    let Some(file_name) = path.file_name() else {
        return false;
    };

    if file_name == FILE_WITH_PREV_TESTS_FAILED {
        return true;
    }

    let Some(stem) = file_name.strip_suffix(".json") else {
        return false;
    };

    let Some((prefix, version)) = stem.rsplit_once("_v") else {
        return false;
    };
    let Some((sanitized_url, block_number)) = prefix.rsplit_once('_') else {
        return false;
    };

    !sanitized_url.is_empty()
        && !block_number.is_empty()
        && block_number.chars().all(|char| char.is_ascii_digit())
        && version.split('_').count() >= 3
        && version
            .split('_')
            .all(|segment| !segment.is_empty() && segment.chars().all(|char| char.is_ascii_digit()))
}

#[cfg(test)]
mod tests {
    use super::is_snfoundry_cache_file;
    use camino::Utf8Path;

    #[test]
    fn recognizes_prev_failed_tests_file() {
        assert!(is_snfoundry_cache_file(Utf8Path::new(".prev_tests_failed")));
    }

    #[test]
    fn recognizes_rpc_cache_file() {
        assert!(is_snfoundry_cache_file(Utf8Path::new(
            "http_111_222_333_444_5050_123_v0_1_0.json"
        )));
    }

    #[test]
    fn rejects_file_without_json_extension() {
        assert!(!is_snfoundry_cache_file(Utf8Path::new(
            "http_111_222_333_444_5050_123_v0_1_0.txt"
        )));
    }

    #[test]
    fn rejects_file_with_non_numeric_block_number() {
        assert!(!is_snfoundry_cache_file(Utf8Path::new(
            "http_111_222_333_444_5050_latest_v0_1_0.json"
        )));
    }

    #[test]
    fn rejects_file_with_incomplete_version() {
        assert!(!is_snfoundry_cache_file(Utf8Path::new(
            "http_111_222_333_444_5050_123_v0_1.json"
        )));
    }

    #[test]
    fn rejects_file_with_empty_sanitized_url() {
        assert!(!is_snfoundry_cache_file(Utf8Path::new("_123_v0_1_0.json")));
    }
}
