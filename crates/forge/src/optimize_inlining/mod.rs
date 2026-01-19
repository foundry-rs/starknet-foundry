mod args;
mod contract_size;
mod optimizer;
mod paths;
mod runner;

pub use args::OptimizeInliningArgs;

use crate::ExitStatus;
use anyhow::{Context, Result, bail};
use camino::Utf8PathBuf;
use foundry_ui::UI;
use optimizer::Optimizer;
use paths::copy_project_to_temp_dir;
use scarb_api::manifest::ManifestEditor;
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
        args.max_contract_program_len
    ));

    let original_metadata = metadata_with_opts(MetadataOpts {
        profile: profile.clone(),
        ..MetadataOpts::default()
    })?;

    let workspace_root = &original_metadata.workspace.root;
    let target_dir = &original_metadata
        .target_dir
        .unwrap_or(workspace_root.join("target"));

    let temp_dir = copy_project_to_temp_dir(workspace_root)?;
    let temp_path = Utf8PathBuf::try_from(temp_dir.path().to_path_buf())
        .context("Temporary directory path is not valid UTF-8")?;

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
