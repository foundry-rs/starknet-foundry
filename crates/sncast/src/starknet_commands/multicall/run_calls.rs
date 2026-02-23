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

pub async fn run_calls(
    tokens: &[String],
    multicall: &Multicall,
    provider: &JsonRpcClient<HttpTransport>,
    account: &SingleOwnerAccount<&JsonRpcClient<HttpTransport>, LocalWallet>,
    wait_config: WaitForTx,
    ui: &UI,
) -> Result<MulticallRunResponse> {
    let allowed_commands = ["deploy".to_string(), "invoke".to_string()];
    let command_groups = extract_commands_groups(tokens, "/", &allowed_commands);

    let mut contracts_registry = ContractsRegistry::new(provider);
    let mut calls = vec![];

    for group in command_groups {
        if group.is_empty() {
            continue;
        }

        let cmd_name = &group[0];
        let cmd_args = &group[1..];

        match cmd_name.as_str() {
            "deploy" => {
                let deploy = parse_args::<MulticallDeploy>(cmd_name, cmd_args)?;
                let call = deploy.build_call(account, &mut contracts_registry).await?;
                calls.push(call);
            }
            "invoke" => {
                let invoke = parse_args::<MulticallInvoke>(cmd_name, cmd_args)?;
                let call = invoke.build_call(&mut contracts_registry).await?;
                calls.push(call);
            }
            _ => bail!("Unknown multicall command: '{cmd_name}'. Expected 'deploy' or 'invoke'."),
        }
    }

    if calls.is_empty() {
        bail!("No valid multicall commands found to execute. Please check the provided commands.");
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

/// Groups tokens into separate command groups based on the provided separator and allowed commands.
fn extract_commands_groups(
    tokens: &[String],
    separator: &str,
    commands: &[String],
) -> Vec<Vec<String>> {
    let mut all_groups = Vec::new();
    let mut current_group = Vec::new();

    for (i, token) in tokens.iter().enumerate() {
        if token == separator {
            let next_index = i + 1;
            let is_at_end = next_index == tokens.len();
            let next_is_command = !is_at_end && commands.contains(&tokens[next_index]);

            if is_at_end || next_is_command {
                if !current_group.is_empty() {
                    all_groups.push(current_group);
                    current_group = Vec::new();
                }
                continue;
            }
        }

        current_group.push(token.clone());
    }

    if !current_group.is_empty() {
        all_groups.push(current_group);
    }

    all_groups
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_commands_groups() {
        let tokens = vec![
            "deploy".to_string(),
            "--class-hash".to_string(),
            "0x123".to_string(),
            "/".to_string(),
            "invoke".to_string(),
            "--contract-address".to_string(),
            "0xabc".to_string(),
            "--function".to_string(),
            "my_function".to_string(),
            "/".to_string(),
            "deploy".to_string(),
            "--class-hash".to_string(),
            "0x456".to_string(),
        ];
        let allowed_commands = vec!["deploy".to_string(), "invoke".to_string()];
        let groups = extract_commands_groups(&tokens, "/", &allowed_commands);
        assert_eq!(
            groups,
            vec![
                vec![
                    "deploy".to_string(),
                    "--class-hash".to_string(),
                    "0x123".to_string()
                ],
                vec![
                    "invoke".to_string(),
                    "--contract-address".to_string(),
                    "0xabc".to_string(),
                    "--function".to_string(),
                    "my_function".to_string()
                ],
                vec![
                    "deploy".to_string(),
                    "--class-hash".to_string(),
                    "0x456".to_string()
                ]
            ]
        );
    }
}
