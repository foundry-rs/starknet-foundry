mod args;
mod contract_size;
mod manifest;
mod optimizer;
mod paths;
mod runner;

pub use args::OptimizeInliningArgs;

use crate::ExitStatus;
use anyhow::{Context, Result, bail};
use camino::Utf8PathBuf;
use foundry_ui::UI;
use manifest::ManifestEditor;
use optimizer::Optimizer;
use paths::copy_project_to_temp_dir;
use scarb_api::metadata::{MetadataOpts, metadata_with_opts};
use std::sync::Arc;

pub fn optimize_inlining(
    args: &OptimizeInliningArgs,
    cores: usize,
    ui: &Arc<UI>,
) -> Result<ExitStatus> {
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
    let target_dir = &original_metadata
        .target_dir
        .unwrap_or(workspace_root.join("target"));
    ui.println(&"Copying project to temporary directory...".to_string());

    let temp_dir = copy_project_to_temp_dir(workspace_root)?;
    let temp_path = Utf8PathBuf::try_from(temp_dir.path().to_path_buf())
        .context("Temporary directory path is not valid UTF-8")?;

    ui.println(&format!("Working in: {temp_path}"));

    let scarb_metadata = metadata_with_opts(MetadataOpts {
        profile: profile.clone(),
        current_dir: Some(temp_path.clone().into()),
        ..MetadataOpts::default()
    })?;

    let manifest_editor = ManifestEditor::new(&original_metadata.runtime_manifest);

    let mut optimizer = Optimizer::new(args, &scarb_metadata);
    let optimization_result = optimizer.optimize(args, cores, ui);

    ui.print_blank_line();
    ui.println(&"Optimization Results:".to_string());
    optimizer.print_results_table(ui);

    let workspace_name = workspace_root
        .file_name()
        .map(|n| format!("{n}_"))
        .unwrap_or_default();
    let graph_path = target_dir.join(format!(
        "{workspace_name}optimization_results_l_{}_h_{}_s_{}.png",
        args.min_threshold, args.max_threshold, args.step
    ));
    create_output_dir::create_output_dir(target_dir.as_std_path())?;
    if let Err(e) = optimizer.save_results_graph(&graph_path, ui) {
        ui.eprintln(&format!("Warning: Failed to save graph: {e}"));
    }

    match optimization_result {
        Ok(_) => {
            let profile_name = profile.unwrap_or_else(|| "dev".to_string());
            let optimal = if args.gas {
                Some(optimizer.find_best_result_by_gas()?)
            } else if args.size {
                Some(optimizer.find_best_result_by_contract_size()?)
            } else {
                None
            };

            if let Some(optimal) = optimal {
                manifest_editor.set_inlining_strategy(optimal.threshold, &profile_name)?;
                ui.println(&format!(
                    "Updated Scarb.toml with inlining-strategy = {}",
                    optimal.threshold
                ));
            } else {
                ui.println(
                    &"Scarb.toml not modified. Use --gas or --size to apply a threshold."
                        .to_string(),
                );
            }

            Ok(ExitStatus::Success)
        }
        Err(e) => {
            bail!(format!("Optimization failed: {e}"));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::paths::{normalize_utf8_path_lexically, rewrite_manifest_paths_to_absolute};
    use anyhow::Result;
    use camino::Utf8PathBuf;
    use indoc::indoc;
    use std::fs;
    use toml_edit::DocumentMut;

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

        let manifest_content = indoc! {r#"
            [package]
            name = "package_a"
            version = "0.1.0"

            [workspace]

            [workspace.dependencies]
            ws_local_dep = { path = "../ws_dep_b" }
            ws_external_dep = { path = "../../../external/ws_dep_c" }

            [dependencies]
            local_dep = { path = "../dep_b" }
            external_dep = { path = "../../../external/dep_c" }

            [[target.test]]
            source-path = "./tests/tests.cairo"

            [[target.starknet-contract]]
            casm = false
            sierra = false
        "#};

        fs::write(original_package.join("Scarb.toml"), manifest_content)?;
        fs::write(copied_package.join("Scarb.toml"), manifest_content)?;

        rewrite_manifest_paths_to_absolute(&original_workspace, &copied_workspace)?;

        let rewritten_manifest = fs::read_to_string(copied_package.join("Scarb.toml"))?;
        let rewritten_doc = rewritten_manifest.parse::<DocumentMut>()?;

        let expected_external_dep_path =
            normalize_utf8_path_lexically(&original_package.join("../../../external/dep_c"))
                .to_string();
        let expected_ws_external_dep_path =
            normalize_utf8_path_lexically(&original_package.join("../../../external/ws_dep_c"))
                .to_string();

        assert_eq!(
            rewritten_doc["dependencies"]["local_dep"]["path"].as_str(),
            Some("../dep_b")
        );
        assert_eq!(
            rewritten_doc["dependencies"]["external_dep"]["path"].as_str(),
            Some(expected_external_dep_path.as_str())
        );
        assert_eq!(
            rewritten_doc["workspace"]["dependencies"]["ws_local_dep"]["path"].as_str(),
            Some("../ws_dep_b")
        );
        assert_eq!(
            rewritten_doc["workspace"]["dependencies"]["ws_external_dep"]["path"].as_str(),
            Some(expected_ws_external_dep_path.as_str())
        );
        assert_eq!(
            rewritten_doc["target"]["test"][0]["source-path"].as_str(),
            Some("./tests/tests.cairo")
        );
        assert_eq!(
            rewritten_doc["target"]["starknet-contract"][0]["casm"].as_bool(),
            Some(true)
        );
        assert_eq!(
            rewritten_doc["target"]["starknet-contract"][0]["sierra"].as_bool(),
            Some(true)
        );

        Ok(())
    }
}
