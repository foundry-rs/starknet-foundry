mod args;
mod contract_size;
mod manifest;
mod optimizer;
mod runner;

pub use args::OptimizeInliningArgs;

use crate::ExitStatus;
use anyhow::{Context, Result, bail};
use camino::{Utf8Path, Utf8PathBuf};
use foundry_ui::UI;
use manifest::ManifestEditor;
use optimizer::Optimizer;
use scarb_api::metadata::{MetadataOpts, metadata_with_opts};
use std::fs;
use std::sync::Arc;
use toml_edit::{DocumentMut, Item, Value, value};

fn copy_project_to_temp_dir(workspace_root: &camino::Utf8Path) -> Result<tempfile::TempDir> {
    let temp_dir = tempfile::TempDir::new().context("Failed to create temporary directory")?;

    let options = fs_extra::dir::CopyOptions::new().content_only(true);

    fs_extra::dir::copy(workspace_root, temp_dir.path(), &options)
        .context("Failed to copy project to temporary directory")?;

    let copied_workspace_root = Utf8PathBuf::from_path_buf(temp_dir.path().to_path_buf())
        .map_err(|_| anyhow::anyhow!("Temporary directory path is not valid UTF-8"))?;
    rewrite_manifest_paths_to_absolute(workspace_root, &copied_workspace_root)?;

    Ok(temp_dir)
}

fn rewrite_manifest_paths_to_absolute(
    original_workspace_root: &Utf8Path,
    copied_workspace_root: &Utf8Path,
) -> Result<()> {
    rewrite_manifest_paths_in_dir(
        original_workspace_root,
        copied_workspace_root,
        copied_workspace_root,
    )
}

fn rewrite_manifest_paths_in_dir(
    original_workspace_root: &Utf8Path,
    copied_workspace_root: &Utf8Path,
    dir: &Utf8Path,
) -> Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = Utf8PathBuf::from_path_buf(entry.path())
            .map_err(|_| anyhow::anyhow!("Workspace path is not valid UTF-8"))?;

        if path.is_dir() {
            rewrite_manifest_paths_in_dir(original_workspace_root, copied_workspace_root, &path)?;
            continue;
        }

        if path.file_name() != Some("Scarb.toml") {
            continue;
        }

        let relative_manifest_path = path
            .strip_prefix(copied_workspace_root)
            .context("Copied manifest path is outside copied workspace root")?;
        let original_manifest_path = original_workspace_root.join(relative_manifest_path);
        let original_manifest_dir = original_manifest_path
            .parent()
            .context("Manifest path has no parent directory")?;

        rewrite_single_manifest_paths_to_absolute(&path, original_manifest_dir)?;
    }

    Ok(())
}

fn rewrite_single_manifest_paths_to_absolute(
    manifest_path: &Utf8Path,
    original_manifest_dir: &Utf8Path,
) -> Result<()> {
    let content = fs::read_to_string(manifest_path)?;
    let mut doc = content
        .parse::<DocumentMut>()
        .context("Failed to parse copied Scarb.toml")?;

    let changed = rewrite_item_paths_to_absolute(doc.as_item_mut(), original_manifest_dir, None);
    if changed {
        fs::write(manifest_path, doc.to_string())?;
    }

    Ok(())
}

fn rewrite_item_paths_to_absolute(
    item: &mut Item,
    original_manifest_dir: &Utf8Path,
    key_hint: Option<&str>,
) -> bool {
    let mut changed = false;

    match item {
        Item::Table(table) => {
            for (key, nested_item) in table.iter_mut() {
                changed |= rewrite_item_paths_to_absolute(
                    nested_item,
                    original_manifest_dir,
                    Some(key.get()),
                );
            }
        }
        Item::ArrayOfTables(array_of_tables) => {
            for table in array_of_tables.iter_mut() {
                for (key, nested_item) in table.iter_mut() {
                    changed |= rewrite_item_paths_to_absolute(
                        nested_item,
                        original_manifest_dir,
                        Some(key.get()),
                    );
                }
            }
        }
        Item::Value(value_item) => {
            changed |= rewrite_value_paths_to_absolute(value_item, original_manifest_dir, key_hint);
        }
        Item::None => {}
    }

    changed
}

fn rewrite_value_paths_to_absolute(
    value_item: &mut Value,
    original_manifest_dir: &Utf8Path,
    key_hint: Option<&str>,
) -> bool {
    let mut changed = false;

    if key_hint.is_some_and(is_path_key) {
        changed |= rewrite_value_if_relative_path(value_item, original_manifest_dir);
    }

    match value_item {
        Value::InlineTable(inline_table) => {
            for (key, inline_value) in inline_table.iter_mut() {
                changed |= rewrite_value_paths_to_absolute(
                    inline_value,
                    original_manifest_dir,
                    Some(key.get()),
                );
            }
        }
        Value::Array(array) => {
            for nested_value in array.iter_mut() {
                changed |=
                    rewrite_value_paths_to_absolute(nested_value, original_manifest_dir, None);
            }
        }
        _ => {}
    }

    changed
}

fn rewrite_value_if_relative_path(
    value_item: &mut Value,
    original_manifest_dir: &Utf8Path,
) -> bool {
    match value_item {
        Value::String(path) => {
            let path_str = path.value();
            if let Some(absolute_path) = absolutize_path(path_str, original_manifest_dir) {
                *value_item = value(absolute_path).into_value().unwrap();
                return true;
            }
            false
        }
        Value::Array(array) => {
            let mut changed = false;
            for nested_value in array.iter_mut() {
                if let Value::String(path) = nested_value {
                    let path_str = path.value();
                    if let Some(absolute_path) = absolutize_path(path_str, original_manifest_dir) {
                        *nested_value = value(absolute_path).into_value().unwrap();
                        changed = true;
                    }
                }
            }
            changed
        }
        _ => false,
    }
}

fn absolutize_path(path: &str, original_manifest_dir: &Utf8Path) -> Option<String> {
    let utf8_path = Utf8Path::new(path);
    if utf8_path.is_absolute() {
        None
    } else {
        Some(original_manifest_dir.join(utf8_path).to_string())
    }
}

fn is_path_key(key: &str) -> bool {
    key == "path" || key.ends_with("-path")
}

#[cfg(test)]
mod tests {
    use super::rewrite_manifest_paths_to_absolute;
    use anyhow::Result;
    use camino::Utf8PathBuf;
    use std::fs;

    #[test]
    fn rewrites_relative_manifest_paths_to_absolute_with_original_manifest_as_base() -> Result<()> {
        let temp_dir = tempfile::tempdir()?;
        let root = Utf8PathBuf::from_path_buf(temp_dir.path().to_path_buf())
            .map_err(|_| anyhow::anyhow!("Temporary path is not valid UTF-8"))?;

        let original_workspace = root.join("original-workspace");
        let copied_workspace = root.join("copied-workspace");
        let original_package = original_workspace.join("crates/package_a");
        let copied_package = copied_workspace.join("crates/package_a");

        fs::create_dir_all(&original_package)?;
        fs::create_dir_all(&copied_package)?;

        let manifest_content = r#"
[package]
name = "package_a"
version = "0.1.0"

[dependencies]
local_dep = { path = "../dep_b" }

[[target.test]]
source-path = "./tests/tests.cairo"
"#;

        fs::write(original_package.join("Scarb.toml"), manifest_content)?;
        fs::write(copied_package.join("Scarb.toml"), manifest_content)?;

        rewrite_manifest_paths_to_absolute(&original_workspace, &copied_workspace)?;

        let rewritten_manifest = fs::read_to_string(copied_package.join("Scarb.toml"))?;

        let expected_dep_path = original_package.join("../dep_b").to_string();
        let expected_source_path = original_package.join("./tests/tests.cairo").to_string();

        assert!(rewritten_manifest.contains(&format!("path = \"{expected_dep_path}\"")));
        assert!(rewritten_manifest.contains(&format!("source-path = \"{expected_source_path}\"")));

        Ok(())
    }
}

pub fn optimize_inlining(args: OptimizeInliningArgs, ui: Arc<UI>) -> Result<ExitStatus> {
    args.validate()?;

    let profile = args.test_args.scarb_args.profile.specified();

    ui.println(&format!(
        "Starting inlining strategy optimization...\n\
         Search range: {} to {}, step: {}, max contract size: {} bytes, max felts: {}",
        args.min_threshold,
        args.max_threshold,
        args.step,
        args.max_contract_size,
        args.max_contract_felts
    ));

    let original_metadata = metadata_with_opts(MetadataOpts {
        profile: profile.clone(),
        ..MetadataOpts::default()
    })?;

    let workspace_root = &original_metadata.workspace.root;
    ui.println(&format!("Copying project to temporary directory..."));

    let _temp_dir = copy_project_to_temp_dir(workspace_root)?;
    let temp_path = Utf8PathBuf::try_from(_temp_dir.path().to_path_buf())
        .context("Temporary directory path is not valid UTF-8")?;

    ui.println(&format!("Working in: {temp_path}"));

    let scarb_metadata = metadata_with_opts(MetadataOpts {
        profile: profile.clone(),
        current_dir: Some(temp_path.clone().into()),
        ..MetadataOpts::default()
    })?;

    let manifest_editor = ManifestEditor::new(&original_metadata.runtime_manifest)?;

    let mut optimizer = Optimizer::new(&args, &scarb_metadata);
    let optimization_result = optimizer.optimize(&args, &ui);

    ui.print_blank_line();
    ui.println(&"Optimization Results:".to_string());
    optimizer.print_results_table(&ui);

    let graph_path = workspace_root.join("optimization_results.png");
    if let Err(e) = optimizer.save_results_graph(&graph_path, &ui) {
        ui.eprintln(&format!("Warning: Failed to save graph: {e}"));
    }

    match optimization_result {
        Ok(optimal) => {
            ui.print_blank_line();
            ui.println(&format!(
                "Optimal threshold: {} (gas: {}, max contract size: {} bytes, max felts: {})",
                optimal.threshold,
                optimal.total_gas.total(),
                optimal.max_contract_size,
                optimal.max_contract_felts
            ));

            let profile_name = profile.unwrap_or_else(|| "dev".to_string());
            if !args.dry_run {
                manifest_editor.set_inlining_strategy(optimal.threshold, &profile_name)?;
                ui.println(&format!(
                    "Updated Scarb.toml with inlining-strategy = {}",
                    optimal.threshold
                ));
            } else {
                ui.println(&"Dry run - Scarb.toml not modified".to_string());
            }

            Ok(ExitStatus::Success)
        }
        Err(e) => {
            bail!(format!("Optimization failed: {e}"));
            Ok(ExitStatus::Failure)
        }
    }
}
