use anyhow::Result;
use clap::{Args, Subcommand};
use serde::Serialize;
use serde_json::{Value, json};

pub mod contract_registry;
pub mod deploy;
pub mod invoke;
pub mod new;
pub mod run;

use crate::starknet_commands::multicall::contract_registry::ContractRegistry;
use crate::{Arguments, process_command_result, starknet_commands};
use foundry_ui::Message;
use new::New;
use run::Run;
use sncast::response::ui::UI;
use sncast::with_account;
use sncast::{
    WaitForTx, get_account,
    helpers::{configuration::CastConfig, constants::DEFAULT_MULTICALL_CONTENTS},
    response::explorer_link::block_explorer_link_if_allowed,
};
use starknet_rust::providers::Provider;

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
) -> Result<()> {
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

            let account = get_account(&config, &provider, &run.rpc, ui).await?;
            let result = with_account!(&account, |account| {
                starknet_commands::multicall::run::run(
                    run.clone(),
                    account,
                    &provider,
                    wait_config,
                    ui,
                )
                .await
            });

            let block_explorer_link =
                block_explorer_link_if_allowed(&result, provider.chain_id().await?, &config).await;
            process_command_result("multicall run", result, ui, block_explorer_link);
            Ok(())
        }
    }
}

/// Replaces arguments that reference user-defined ids with their corresponding values from the contract registry.
pub fn replaced_arguments(
    arguments: &Arguments,
    contract_registry: &ContractRegistry,
) -> Result<Arguments> {
    Ok(match (&arguments.calldata, &arguments.arguments) {
        (Some(calldata), None) => {
            let replaced_calldata = calldata
                .iter()
                .map(|input| {
                    if let Some(address) = contract_registry.get_address_by_id(input) {
                        Ok(address.to_string())
                    } else {
                        Ok(input.clone())
                    }
                })
                .collect::<Result<Vec<String>>>()?;
            Arguments {
                calldata: Some(replaced_calldata),
                arguments: None,
            }
        }
        (None, _) => arguments.clone(),
        (Some(_), Some(_)) => anyhow::bail!(
            "Invalid arguments: both `calldata` and `arguments` are set. Please provide only one."
        ),
    })
}
