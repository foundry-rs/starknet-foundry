use anyhow::{Context, Result};
use clap::{Args, Subcommand};
use serde::Serialize;
use serde_json::{Value, json};

pub mod contract_registry;
pub mod deploy;
pub mod execute;
pub mod invoke;
pub mod mode;
pub mod new;
pub mod run;

use crate::starknet_commands::multicall::contract_registry::ContractRegistry;
use crate::starknet_commands::multicall::execute::Execute;
use crate::starknet_commands::multicall::mode::MulticallMode;
use crate::{Arguments, process_command_result, starknet_commands};
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
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    Run(Box<Run>),
    New(New),
    Execute(Execute),
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

            let account =
                get_account(&config, &provider, &run.rpc, config.keystore.as_ref(), ui).await?;
            let result = starknet_commands::multicall::run::run(
                run.clone(),
                &account,
                &provider,
                wait_config,
                run.fee_args.clone(),
                ui,
            )
            .await;

            let block_explorer_link =
                block_explorer_link_if_allowed(&result, provider.chain_id().await?, &config).await;
            process_command_result("multicall run", result, ui, block_explorer_link);
            Ok(())
        }
        starknet_commands::multicall::Commands::Execute(execute) => {
            let provider = execute.rpc.get_provider(&config, ui).await?;
            let account = get_account(
                &config,
                &provider,
                &execute.rpc,
                config.keystore.as_ref(),
                ui,
            )
            .await?;

            let result = starknet_commands::multicall::execute::execute(
                execute.clone(),
                &account,
                &provider,
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

/// Replaces arguments that reference user-defined ids with their corresponding values from the contract registry.
pub fn replaced_arguments(
    arguments: &Arguments,
    contract_registry: &ContractRegistry,
    mode: MulticallMode,
) -> Result<Arguments> {
    Ok(match (&arguments.calldata, &arguments.arguments) {
        (Some(calldata), None) => {
            let replaced_calldata = calldata
                .iter()
                .map(|input| {
                    Ok(resolve_contract_address(input, contract_registry, mode)
                        .context("Failed to resolve contract address")?
                        .to_string())
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

/// Resolves a contract address from a string that can either be a direct address
/// or an id referencing a previously defined contract in the registry, depending on the mode.
pub fn resolve_contract_address(
    contract_address: &str,
    contracts: &ContractRegistry,
    mode: MulticallMode,
) -> Result<Felt> {
    let parse_fallback = || contract_address.parse::<Felt>().map_err(Into::into);

    match mode {
        MulticallMode::File => {
            contracts
            .get_address_by_id(contract_address)
            .map_or_else(parse_fallback, Ok)
        },
        MulticallMode::Cli => match mode.id_key(contract_address) {
            Some(id) => contracts.get_address_by_id(id).ok_or_else(|| {
                anyhow::anyhow!(
                    "No contract address found for id: {id}. Ensure the referenced id is defined in a previous step."
                )
            }),
            None => parse_fallback(),
        },
    }
}
