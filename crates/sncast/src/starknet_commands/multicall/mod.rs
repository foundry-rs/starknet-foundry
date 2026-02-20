use clap::{Args, Subcommand};
use serde::Serialize;
use serde_json::{Value, json};
use sncast::helpers::fee::FeeArgs;
use sncast::helpers::rpc::RpcArgs;

pub mod ctx;
pub mod deploy;
pub mod invoke;
pub mod new;
pub mod run;
pub mod run_with_calls;

use crate::{process_command_result, starknet_commands};
use foundry_ui::Message;
use new::New;
use run::Run;
use sncast::response::ui::UI;
use sncast::{
    WaitForTx, get_account,
    helpers::{configuration::CastConfig, constants::DEFAULT_MULTICALL_CONTENTS},
    response::explorer_link::block_explorer_link_if_allowed,
};
use starknet_rust::providers::Provider;
use starknet_types_core::felt::Felt;

#[derive(Args)]
#[command(about = "Execute multiple calls at once", long_about = None)]
pub struct Multicall {
    #[command(flatten)]
    pub fee_args: FeeArgs,

    #[command(flatten)]
    pub rpc: RpcArgs,

    /// Nonce of the transaction. If not provided, nonce will be set automatically
    #[arg(short, long)]
    pub nonce: Option<Felt>,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    Run(Box<Run>),
    New(New),
    #[command(external_subcommand)]
    Calls(Vec<String>),
}

pub async fn multicall(
    multicall: Multicall,
    config: CastConfig,
    ui: &UI,
    wait_config: WaitForTx,
) -> anyhow::Result<()> {
    #[derive(Serialize)]
    struct MulticallMessage {
        file_contents: String,
    }

    impl Message for MulticallMessage {
        fn text(&self) -> String {
            self.file_contents.clone()
        }

        fn json(&self) -> Value {
            json!(self)
        }
    }

    match &multicall.command {
        starknet_commands::multicall::Commands::New(new) => {
            if let Some(output_path) = &new.output_path {
                let result = starknet_commands::multicall::new::write_empty_template(
                    output_path,
                    new.overwrite,
                );

                process_command_result("multicall new", result, ui, None);
            } else {
                ui.print_message(
                    "multicall_new",
                    MulticallMessage {
                        file_contents: DEFAULT_MULTICALL_CONTENTS.to_string(),
                    },
                );
            }
            Ok(())
        }
        starknet_commands::multicall::Commands::Run(run) => {
            let provider = run.rpc.get_provider(&config, ui).await?;

            let account =
                get_account(&config, &provider, &run.rpc, config.keystore.as_ref(), ui).await?;
            let result = starknet_commands::multicall::run::run(
                run.clone(),
                &account,
                wait_config,
                &provider,
                ui,
            )
            .await;

            let block_explorer_link =
                block_explorer_link_if_allowed(&result, provider.chain_id().await?, &config).await;
            process_command_result("multicall run", result, ui, block_explorer_link);
            Ok(())
        }
        starknet_commands::multicall::Commands::Calls(tokens) => {
            let provider = multicall.rpc.get_provider(&config, ui).await?;
            let account = get_account(
                &config,
                &provider,
                &multicall.rpc,
                config.keystore.as_ref(),
                ui,
            )
            .await?;

            let result = starknet_commands::multicall::run_with_calls::run_with_calls(
                tokens,
                &multicall,
                &provider,
                &account,
                wait_config,
                ui,
            )
            .await;
            let block_explorer_link =
                block_explorer_link_if_allowed(&result, provider.chain_id().await?, &config).await;
            process_command_result("multicall", result, ui, block_explorer_link);
            Ok(())
        }
    }
}
