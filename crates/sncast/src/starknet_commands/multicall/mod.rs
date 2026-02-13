use anyhow::ensure;
use clap::{Args, FromArgMatches, Subcommand};
use serde::Serialize;
use serde_json::{Value, json};
use sncast::helpers::fee::FeeArgs;
use sncast::helpers::rpc::RpcArgs;
use sncast::response::errors::handle_starknet_command_error;

mod ctx;
mod deploy;
mod invoke;
mod new;
mod run;

use crate::starknet_commands::invoke::{InvokeArgs, execute_calls};
use crate::starknet_commands::multicall::ctx::MulticallCtx;
use crate::starknet_commands::multicall::deploy::MulticallDeploy;
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

/// `[Args]` wrapper to match `T` variants recursively in `U`.
#[derive(Debug, Clone)]
pub struct ReClap<T, U>
where
    T: Args,
    U: Subcommand,
{
    /// Specific Variant.
    pub inner: T,
    /// Enum containing `Self<T>` variants, in other words possible follow-up commands.
    pub next: Option<Box<U>>,
}

impl<T, U> Args for ReClap<T, U>
where
    T: Args,
    U: Subcommand,
{
    fn augment_args(cmd: clap::Command) -> clap::Command {
        T::augment_args(cmd).defer(|cmd| U::augment_subcommands(cmd.disable_help_subcommand(true)))
    }
    fn augment_args_for_update(_cmd: clap::Command) -> clap::Command {
        unimplemented!()
    }
}

impl<T, U> FromArgMatches for ReClap<T, U>
where
    T: Args,
    U: Subcommand,
{
    fn from_arg_matches(matches: &clap::ArgMatches) -> Result<Self, clap::Error> {
        let inner = T::from_arg_matches(matches)?;
        let next = if let Some((_name, _sub)) = matches.subcommand() {
            Some(U::from_arg_matches(matches)?)
        } else {
            None
        };
        Ok(Self {
            inner,
            next: next.map(Box::new),
        })
    }

    fn update_from_arg_matches(&mut self, _matches: &clap::ArgMatches) -> Result<(), clap::Error> {
        unimplemented!()
    }
}

#[derive(Args)]
#[command(about = "Execute multiple calls at once", long_about = None)]
pub struct Multicall {
    #[command(flatten)]
    pub fee_args: FeeArgs,

    #[command(flatten)]
    pub rpc: RpcArgs,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    Run(Box<Run>),
    New(New),
    Deploy(Box<ReClap<MulticallDeploy, ChainCommands>>),
    Invoke(Box<ReClap<InvokeArgs, ChainCommands>>),
}

#[derive(Debug, Subcommand)]
pub enum ChainCommands {
    #[command(hide = true)]
    Deploy(Box<ReClap<MulticallDeploy, ChainCommands>>),

    #[command(hide = true)]
    Invoke(Box<ReClap<InvokeArgs, ChainCommands>>),
}

enum CmdState {
    Top(Commands),
    Chain(ChainCommands),
}

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

pub async fn multicall(
    multicall: Multicall,
    config: CastConfig,
    ui: &UI,
    wait_config: WaitForTx,
) -> anyhow::Result<()> {
    let mut ctx = MulticallCtx::default();

    let mut current = Some(CmdState::Top(multicall.command));

    while let Some(state) = current.take() {
        match state {
            CmdState::Top(cmd) => match cmd {
                Commands::Deploy(re_clap) => {
                    let provider = multicall.rpc.get_provider(&config, ui).await?;
                    deploy::deploy(re_clap.inner, &mut ctx, &provider).await?;
                    current = re_clap.next.map(|b| CmdState::Chain(*b));
                }
                Commands::Invoke(re_clap) => {
                    let provider = multicall.rpc.get_provider(&config, ui).await?;
                    invoke::invoke(re_clap.inner, &mut ctx, &provider).await?;
                    current = re_clap.next.map(|b| CmdState::Chain(*b));
                }

                Commands::New(new) => {
                    ensure!(ctx.calls().is_empty());

                    if let Some(output_path) = &new.output_path {
                        let result = starknet_commands::multicall::new::write_empty_template(
                            output_path,
                            new.overwrite,
                        );

                        process_command_result("multicall new", result, ui, None);
                    } else {
                        ui.print_message(
                            "multicall new",
                            MulticallMessage {
                                file_contents: DEFAULT_MULTICALL_CONTENTS.to_string(),
                            },
                        );
                    }
                    return Ok(());
                }
                Commands::Run(run) => {
                    ensure!(ctx.calls().is_empty());

                    let provider = run.rpc.get_provider(&config, ui).await?;
                    let account =
                        get_account(&config, &provider, &run.rpc, config.keystore.as_ref(), ui)
                            .await?;

                    let result = starknet_commands::multicall::run::run(
                        run.clone(),
                        &account,
                        wait_config,
                        ui,
                    )
                    .await;

                    let block_explorer_link = block_explorer_link_if_allowed(
                        &result,
                        provider.chain_id().await?,
                        &config,
                    )
                    .await;
                    process_command_result("multicall run", result, ui, block_explorer_link);

                    return Ok(());
                }
            },

            CmdState::Chain(cmd) => match cmd {
                ChainCommands::Deploy(re_clap) => {
                    let provider = multicall.rpc.get_provider(&config, ui).await?;
                    deploy::deploy(re_clap.inner, &mut ctx, &provider).await?;
                    current = re_clap.next.map(|b| CmdState::Chain(*b));
                }
                ChainCommands::Invoke(re_clap) => {
                    let provider = multicall.rpc.get_provider(&config, ui).await?;
                    invoke::invoke(re_clap.inner, &mut ctx, &provider).await?;
                    current = re_clap.next.map(|b| CmdState::Chain(*b));
                }
            },
        }
    }

    // At this point, we should have processed chained commands
    ensure!(!ctx.calls().is_empty(), "No calls to execute");

    let provider = multicall.rpc.get_provider(&config, ui).await?;
    let account = get_account(
        &config,
        &provider,
        &multicall.rpc,
        config.keystore.as_ref(),
        ui,
    )
    .await?;

    let result = execute_calls(
        &account,
        ctx.calls().to_vec(),
        multicall.fee_args,
        None,
        wait_config,
        ui,
    )
    .await
    .map_err(handle_starknet_command_error);

    let block_explorer_link =
        block_explorer_link_if_allowed(&result, provider.chain_id().await?, &config).await;
    process_command_result("multicall", result, ui, block_explorer_link);

    Ok(())
}
