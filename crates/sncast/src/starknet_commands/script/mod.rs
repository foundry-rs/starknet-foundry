use crate::starknet_commands::script::run::Run;
use crate::{Cli, starknet_commands::script::init::Init};
use crate::{get_cast_config, process_command_result, starknet_commands};
use clap::{Args, Subcommand};
use foundry_ui::UI;
use sncast::helpers::scarb_utils::{
    BuildConfig, assert_manifest_path_exists, build, build_and_load_artifacts,
    get_package_metadata, get_scarb_metadata_with_deps,
};
use sncast::{chain_id_to_network_name, get_chain_id, get_default_state_file_name};
use tokio::runtime::Runtime;

pub mod init;
pub mod run;

#[derive(Args)]
pub struct Script {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    Init(Init),
    Run(Run),
}

pub fn run_script_command(
    cli: &Cli,
    runtime: Runtime,
    script: &Script,
    ui: &UI,
) -> anyhow::Result<()> {
    match &script.command {
        starknet_commands::script::Commands::Init(init) => {
            let result = starknet_commands::script::init::init(init, ui);
            process_command_result("script init", result, ui, None);
        }
        starknet_commands::script::Commands::Run(run) => {
            let manifest_path = assert_manifest_path_exists()?;
            let package_metadata = get_package_metadata(&manifest_path, &run.package)?;

            let config = get_cast_config(cli, ui)?;

            let provider = runtime.block_on(run.rpc.get_provider(&config, ui))?;

            let mut artifacts = build_and_load_artifacts(
                &package_metadata,
                &BuildConfig {
                    scarb_toml_path: manifest_path.clone(),
                    json: cli.json,
                    profile: cli.profile.clone().unwrap_or("dev".to_string()),
                },
                true,
                ui,
            )
            .expect("Failed to build artifacts");
            // TODO(#2042): remove duplicated compilation
            build(
                &package_metadata,
                &BuildConfig {
                    scarb_toml_path: manifest_path.clone(),
                    json: cli.json,
                    profile: "dev".to_string(),
                },
                "dev",
            )
            .expect("Failed to build script");
            let metadata_with_deps = get_scarb_metadata_with_deps(&manifest_path)?;

            let chain_id = runtime.block_on(get_chain_id(&provider))?;
            let state_file_path = if run.no_state_file {
                None
            } else {
                Some(package_metadata.root.join(get_default_state_file_name(
                    &run.script_name,
                    &chain_id_to_network_name(chain_id),
                )))
            };

            let result = starknet_commands::script::run::run(
                &run.script_name,
                &metadata_with_deps,
                &package_metadata,
                &mut artifacts,
                &provider,
                runtime,
                &config,
                state_file_path,
                ui,
            );

            process_command_result("script run", result, ui, None);
        }
    }

    Ok(())
}
