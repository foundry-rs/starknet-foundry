mod args;
mod contract_size;
mod manifest;
mod optimizer;
mod runner;

pub use args::OptimizeInliningArgs;

use crate::ExitStatus;
use anyhow::{Context, Result};
use camino::Utf8PathBuf;
use foundry_ui::UI;
use manifest::ManifestEditor;
use optimizer::Optimizer;
use scarb_api::metadata::{MetadataOpts, metadata_with_opts};
use std::sync::Arc;

fn copy_project_to_temp_dir(workspace_root: &camino::Utf8Path) -> Result<tempfile::TempDir> {
    let temp_dir = tempfile::TempDir::new().context("Failed to create temporary directory")?;

    let options = fs_extra::dir::CopyOptions::new().content_only(true);

    fs_extra::dir::copy(workspace_root, temp_dir.path(), &options)
        .context("Failed to copy project to temporary directory")?;

    Ok(temp_dir)
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
            ui.eprintln(&format!("Optimization failed: {e}"));
            Ok(ExitStatus::Failure)
        }
    }
}
