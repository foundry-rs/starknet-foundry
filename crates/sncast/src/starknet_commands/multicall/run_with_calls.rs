use anyhow::{Context, Result, bail};
use clap::{Args, Command, FromArgMatches};
use sncast::{
    WaitForTx,
    response::{
        errors::handle_starknet_command_error, multicall::run::MulticallRunResponse, ui::UI,
    },
};
use starknet_rust::{
    accounts::SingleOwnerAccount,
    providers::{JsonRpcClient, jsonrpc::HttpTransport},
    signers::LocalWallet,
};

use crate::starknet_commands::{
    invoke::execute_calls,
    multicall::{
        Multicall, contracts_registry::ContractsRegistry, deploy::MulticallDeploy,
        invoke::MulticallInvoke,
    },
};

pub async fn run_with_calls(
    tokens: &[String],
    multicall: &Multicall,
    provider: &JsonRpcClient<HttpTransport>,
    account: &SingleOwnerAccount<&JsonRpcClient<HttpTransport>, LocalWallet>,
    wait_config: WaitForTx,
    ui: &UI,
) -> Result<MulticallRunResponse> {
    let input = tokens.join(" ");
    let commands = input.split('/').map(str::trim).filter(|s| !s.is_empty());

    let mut contracts_registry = ContractsRegistry::new(provider);
    let mut calls = vec![];

    for command in commands {
        let args = shell_words::split(command)?;
        let command = args
            .first()
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("Empty command"))?;

        match command.as_str() {
            "deploy" => {
                let deploy = parse_args::<MulticallDeploy>(&command, &args[1..])?;
                let call = deploy
                    .convert_to_call(account, &mut contracts_registry)
                    .await?;
                calls.push(call);
            }
            "invoke" => {
                let invoke = parse_args::<MulticallInvoke>(&command, &args[1..])?;
                let call = invoke.convert_to_call(&mut contracts_registry).await?;
                calls.push(call);
            }
            _ => {
                bail!("Unknown command: {command}");
            }
        }
    }

    execute_calls(
        account,
        calls,
        multicall.fee_args.clone(),
        multicall.nonce,
        wait_config,
        ui,
    )
    .await
    .map(Into::into)
    .map_err(handle_starknet_command_error)
}

fn parse_args<T>(command_name: &str, tokens: &[String]) -> anyhow::Result<T>
where
    T: Args + FromArgMatches,
{
    let cmd = T::augment_args(Command::new(command_name.to_string()));

    let argv = std::iter::once(command_name.to_string())
        .chain(tokens.iter().cloned())
        .collect::<Vec<_>>();

    let matches = cmd
        .try_get_matches_from(argv)
        .with_context(|| format!("Failed to parse args for `{command_name}`"))?;

    T::from_arg_matches(&matches).map_err(Into::into)
}
