use crate::optimize_inlining::manifest::overwrite_starknet_contract_target_flags;
use anyhow::{Context, Result};
use camino::{Utf8Component, Utf8Path, Utf8PathBuf};
use std::fs;
use toml_edit::{DocumentMut, Item, Value};

// Instead of modifying user project in place (i.e. setting inlining strategy in manifest file), we copy it to a temp dir.
// Note this operation is not a trivial recursive copy, as we take care to properly handle relative paths as well.
pub(super) fn copy_project_to_temp_dir(
    workspace_root: &camino::Utf8Path,
) -> Result<tempfile::TempDir> {
    let temp_dir = tempfile::TempDir::new().context("Failed to create temporary directory")?;

    let options = fs_extra::dir::CopyOptions::new().content_only(true);

    fs_extra::dir::copy(workspace_root, temp_dir.path(), &options)
        .context("Failed to copy project to temporary directory")?;

    let copied_workspace_root = Utf8PathBuf::from_path_buf(temp_dir.path().to_path_buf())
        .map_err(|_| anyhow::anyhow!("Temporary directory path is not valid UTF-8"))?;
    rewrite_manifest_paths_to_absolute(workspace_root, &copied_workspace_root)?;

    Ok(temp_dir)
}

pub(super) fn rewrite_manifest_paths_to_absolute(
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

        rewrite_single_manifest_paths_to_absolute(
            &path,
            original_manifest_dir,
            original_workspace_root,
        )?;
    }

    Ok(())
}

fn rewrite_single_manifest_paths_to_absolute(
    manifest_path: &Utf8Path,
    original_manifest_dir: &Utf8Path,
    original_workspace_root: &Utf8Path,
) -> Result<()> {
    let content = fs::read_to_string(manifest_path)?;
    let mut doc = content
        .parse::<DocumentMut>()
        .context("Failed to parse copied Scarb.toml")?;

    let paths_rewritten = rewrite_dependency_paths_to_absolute(
        &mut doc,
        original_manifest_dir,
        original_workspace_root,
    );
    let target_flags_overwritten = overwrite_starknet_contract_target_flags(&mut doc);

    if paths_rewritten || target_flags_overwritten {
        fs::write(manifest_path, doc.to_string())?;
    }

    Ok(())
}

// Rewrite path dependencies in package and workspace sections.
fn rewrite_dependency_paths_to_absolute(
    doc: &mut DocumentMut,
    original_manifest_dir: &Utf8Path,
    original_workspace_root: &Utf8Path,
) -> bool {
    let mut changed = false;

    if let Some(workspace_dependencies) = doc
        .as_table_mut()
        .get_mut("workspace")
        .and_then(Item::as_table_mut)
        .and_then(|workspace| workspace.get_mut("dependencies"))
    {
        changed |= rewrite_dependency_table_paths_to_absolute(
            workspace_dependencies,
            original_manifest_dir,
            original_workspace_root,
        );
    }

    if let Some(package_dependencies) = doc.as_table_mut().get_mut("dependencies") {
        changed |= rewrite_dependency_table_paths_to_absolute(
            package_dependencies,
            original_manifest_dir,
            original_workspace_root,
        );
    }

    changed
}

fn rewrite_dependency_table_paths_to_absolute(
    dependencies_item: &mut Item,
    original_manifest_dir: &Utf8Path,
    original_workspace_root: &Utf8Path,
) -> bool {
    let Some(dependencies_table) = dependencies_item.as_table_mut() else {
        return false;
    };

    let mut changed = false;
    for (_, dependency_item) in dependencies_table.iter_mut() {
        match dependency_item {
            Item::Value(Value::InlineTable(inline_table)) => {
                if let Some(path_value) = inline_table.get_mut("path") {
                    changed |= rewrite_value_if_relative_path(
                        path_value,
                        original_manifest_dir,
                        original_workspace_root,
                    );
                }
            }
            Item::Table(dependency_table) => {
                if let Some(path_item) = dependency_table.get_mut("path")
                    && let Some(path_value) = path_item.as_value_mut()
                {
                    changed |= rewrite_value_if_relative_path(
                        path_value,
                        original_manifest_dir,
                        original_workspace_root,
                    );
                }
            }
            _ => {}
        }
    }

    changed
}

fn rewrite_value_if_relative_path(
    value_item: &mut Value,
    original_manifest_dir: &Utf8Path,
    original_workspace_root: &Utf8Path,
) -> bool {
    match value_item {
        Value::String(path) => {
            let path_str = path.value();
            if let Some(absolute_path) =
                absolutize_path(path_str, original_manifest_dir, original_workspace_root)
            {
                *value_item = Value::from(absolute_path);
                return true;
            }
            false
        }
        _ => false,
    }
}

fn absolutize_path(
    path: &str,
    original_manifest_dir: &Utf8Path,
    original_workspace_root: &Utf8Path,
) -> Option<String> {
    let utf8_path = Utf8Path::new(path);
    if utf8_path.is_absolute() {
        None
    } else {
        let resolved_path = normalize_utf8_path_lexically(&original_manifest_dir.join(utf8_path));
        if resolved_path.starts_with(original_workspace_root) {
            None
        } else {
            Some(resolved_path.to_string())
        }
    }
}

pub(super) fn normalize_utf8_path_lexically(path: &Utf8PathBuf) -> Utf8PathBuf {
    let mut normalized_parts: Vec<String> = Vec::new();
    let mut prefix: Option<String> = None;
    let is_absolute = path.is_absolute();

    for component in path.components() {
        match component {
            Utf8Component::Prefix(prefix_component) => {
                prefix = Some(prefix_component.as_str().to_string());
            }
            Utf8Component::RootDir | Utf8Component::CurDir => {}
            Utf8Component::ParentDir => {
                if is_absolute || normalized_parts.last().is_some_and(|part| part != "..") {
                    normalized_parts.pop();
                } else {
                    normalized_parts.push("..".to_string());
                }
            }
            Utf8Component::Normal(part) => normalized_parts.push(part.to_string()),
        }
    }

    let mut normalized_path = String::new();
    if let Some(prefix) = prefix {
        normalized_path.push_str(&prefix);
    }
    if is_absolute {
        normalized_path.push('/');
    }
    normalized_path.push_str(&normalized_parts.join("/"));

    if normalized_path.is_empty() {
        ".".into()
    } else {
        normalized_path.into()
    }
}
