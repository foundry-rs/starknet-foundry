use clap::{Args, Subcommand};

pub mod new;
pub mod run;

use foundry_ui::UI;
use new::New;
use run::Run;
use sncast::{
    WaitForTx, get_account,
    helpers::{configuration::CastConfig, constants::DEFAULT_MULTICALL_CONTENTS},
    response::explorer_link::block_explorer_link_if_allowed,
};
use starknet::providers::Provider;

use crate::{process_command_result, starknet_commands};

#[derive(Args)]
#[command(about = "Execute multiple calls at once", long_about = None)]
pub struct Multicall {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    Run(Box<Run>),
    New(New),
}

pub async fn multicall(
    multicall: Multicall,
    config: CastConfig,
    ui: &UI,
    wait_config: WaitForTx,
) -> anyhow::Result<()> {
    match &multicall.command {
        starknet_commands::multicall::Commands::New(new) => {
            if let Some(output_path) = &new.output_path {
                let result = starknet_commands::multicall::new::write_empty_template(
                    output_path,
                    new.overwrite,
                );

                process_command_result("multicall new", result, ui, None);
            } else {
                ui.println(&DEFAULT_MULTICALL_CONTENTS);
            }
            Ok(())
        }
        starknet_commands::multicall::Commands::Run(run) => {
            let provider = run.rpc.get_provider(&config, ui).await?;

            let account = get_account(
                &config.account,
                &config.accounts_file,
                &provider,
                config.keystore.as_ref(),
            )
            .await?;
            let result =
                starknet_commands::multicall::run::run(run.clone(), &account, wait_config, ui)
                    .await;

            let block_explorer_link = block_explorer_link_if_allowed(
                &result,
                provider.chain_id().await?,
                &run.rpc,
                &config,
            );
            process_command_result("multicall run", result, ui, block_explorer_link);
            Ok(())
        }
    }
}
