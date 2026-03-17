use anyhow::{Context, Result, bail};
use clap::{Args, Command, FromArgMatches};
use sncast::{
    WaitForTx,
    helpers::{fee::FeeArgs, rpc::RpcArgs},
    response::{
        errors::handle_starknet_command_error, multicall::run::MulticallRunResponse, ui::UI,
    },
};
use starknet_rust::{
    accounts::SingleOwnerAccount,
    providers::{JsonRpcClient, jsonrpc::HttpTransport},
    signers::LocalWallet,
};
use starknet_types_core::felt::Felt;

use crate::starknet_commands::{
    invoke::execute_calls,
    multicall::{
        contract_registry::ContractRegistry, deploy::MulticallDeploy, invoke::MulticallInvoke,
    },
};

const ALLOWED_MULTICALL_COMMANDS: [&str; 2] = ["deploy", "invoke"];

#[derive(Args, Debug, Clone)]
#[command(about = "Execute a multicall with CLI arguments")]
pub struct Execute {
    #[command(flatten)]
    pub fee_args: FeeArgs,

    #[command(flatten)]
    pub rpc: RpcArgs,

    /// Nonce of the transaction. If not provided, nonce will be set automatically
    #[arg(short, long)]
    pub nonce: Option<Felt>,

    /// The multicall arguments. Subsequent calls should be separated by a '/' token.
    #[arg(allow_hyphen_values = true, num_args = 1..)]
    pub tokens: Vec<String>,
}

pub async fn execute(
    execute: Execute,
    account: &SingleOwnerAccount<&JsonRpcClient<HttpTransport>, LocalWallet>,
    provider: &JsonRpcClient<HttpTransport>,
    wait_config: WaitForTx,
    ui: &UI,
) -> Result<MulticallRunResponse> {
    let command_groups = extract_commands_groups(&execute.tokens, "/", &ALLOWED_MULTICALL_COMMANDS);

    let mut contract_registry = ContractRegistry::new(provider);
    let mut calls = vec![];

    for group in command_groups {
        if group.is_empty() {
            continue;
        }

        let cmd_name = &group[0];
        let cmd_args = &group[1..];

        match cmd_name.as_str() {
            "deploy" => {
                let call = parse_args::<MulticallDeploy>(cmd_name, cmd_args)?
                    .build_call(account, &mut contract_registry)
                    .await?;
                calls.push(call);
            }
            "invoke" => {
                let call = parse_args::<MulticallInvoke>(cmd_name, cmd_args)?
                    .build_call(&mut contract_registry)
                    .await?;
                calls.push(call);
            }
            _ => bail!(
                "Unknown multicall command: '{}'. Allowed commands: {}",
                cmd_name,
                ALLOWED_MULTICALL_COMMANDS.join(", ")
            ),
        }
    }

    if calls.is_empty() {
        bail!("No valid multicall commands found to execute. Please check the provided commands.");
    }

    execute_calls(
        account,
        calls,
        execute.fee_args.clone(),
        execute.nonce,
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
    commands: &[&str],
) -> Vec<Vec<String>> {
    let mut all_groups = Vec::new();
    let mut current_group = Vec::new();

    let mut i = 0;
    while i < tokens.len() {
        let token = &tokens[i];

        if token == separator {
            // Look ahead to find the next non-separator token
            let mut j = i + 1;
            while j < tokens.len() && tokens[j] == separator {
                j += 1;
            }

            let is_at_end = j == tokens.len();
            let next_is_command = !is_at_end && commands.contains(&tokens[j].as_str());

            // If the sequence of separators leads to a command or the end of the input,
            // it acts as a valid boundary.
            if is_at_end || next_is_command {
                if !current_group.is_empty() {
                    all_groups.push(current_group);
                    current_group = Vec::new();
                }
                // Fast-forward the index to skip all consecutive separators
                i = j;
                continue;
            }
        }

        current_group.push(token.clone());
        i += 1;
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
            "deploy",
            "--class-hash",
            "0x123",
            "/",
            "invoke",
            "--contract-address",
            "0xabc",
            "--function",
            "my_function",
            "/",
            "deploy",
            "--class-hash",
            "0x456",
        ]
        .into_iter()
        .map(String::from)
        .collect::<Vec<_>>();

        let groups = extract_commands_groups(&tokens, "/", &ALLOWED_MULTICALL_COMMANDS);
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

    #[test]
    fn test_extract_commands_groups_leading_slash() {
        let tokens = vec!["/", "deploy", "--class-hash", "0x123"]
            .into_iter()
            .map(String::from)
            .collect::<Vec<_>>();

        let groups = extract_commands_groups(&tokens, "/", &ALLOWED_MULTICALL_COMMANDS);
        assert_eq!(
            groups,
            vec![vec![
                "deploy".to_string(),
                "--class-hash".to_string(),
                "0x123".to_string()
            ]]
        );
    }

    #[test]
    fn test_extract_commands_groups_trailing_slash() {
        let tokens = vec!["deploy", "--class-hash", "0x123", "/"]
            .into_iter()
            .map(String::from)
            .collect::<Vec<_>>();

        let groups = extract_commands_groups(&tokens, "/", &ALLOWED_MULTICALL_COMMANDS);
        assert_eq!(
            groups,
            vec![vec![
                "deploy".to_string(),
                "--class-hash".to_string(),
                "0x123".to_string()
            ]]
        );
    }

    #[test]
    fn test_extract_commands_groups_consecutive_slashes() {
        let tokens = vec![
            "deploy",
            "--class-hash",
            "0x123",
            "/",
            "/",
            "invoke",
            "--contract-address",
            "0xabc",
        ]
        .into_iter()
        .map(String::from)
        .collect::<Vec<_>>();

        let groups = extract_commands_groups(&tokens, "/", &ALLOWED_MULTICALL_COMMANDS);
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
                    "0xabc".to_string()
                ]
            ]
        );
    }

    #[test]
    fn test_extract_commands_groups_only_slashes() {
        let tokens = vec!["/", "/", "/"]
            .into_iter()
            .map(String::from)
            .collect::<Vec<_>>();

        let groups = extract_commands_groups(&tokens, "/", &ALLOWED_MULTICALL_COMMANDS);
        let expected: Vec<Vec<String>> = vec![];
        assert_eq!(groups, expected);
    }
}
