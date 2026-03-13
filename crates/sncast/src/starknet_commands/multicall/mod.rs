use anyhow::Result;
use clap::{Args, Subcommand};
use serde::Serialize;
use serde_json::{Value, json};
use sncast::helpers::fee::FeeArgs;
use sncast::helpers::rpc::RpcArgs;

pub mod contract_registry;
pub mod deploy;
pub mod invoke;
pub mod mode;
pub mod new;
pub mod run;
pub mod run_calls;

use crate::starknet_commands::multicall::contract_registry::ContractRegistry;
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
            let provider = multicall.rpc.get_provider(&config, ui).await?;

            let account = get_account(
                &config,
                &provider,
                &multicall.rpc,
                config.keystore.as_ref(),
                ui,
            )
            .await?;
            let result = starknet_commands::multicall::run::run(
                run.clone(),
                &account,
                &provider,
                wait_config,
                multicall.fee_args.clone(),
                multicall.nonce,
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

            let result = starknet_commands::multicall::run_calls::run_calls(
                tokens,
                &provider,
                &account,
                wait_config,
                multicall.fee_args.clone(),
                multicall.nonce,
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
///
/// - For [`MulticallSource::File`], ids are referenced without a prefix (e.g. `deployed_contract`).
/// - For [`MulticallSource::Cli`], ids are referenced with an `@` prefix (e.g. `@deployed_contract`).
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
                    if let Some(id_key) = mode.id_key(input) {
                        Ok(contract_registry
                            .get_address_by_id(id_key)
                            .map_or_else(|| input.clone(), |a| a.to_string()))
                    } else {
                        // For CLI, values without `@` are treated as literals.
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
