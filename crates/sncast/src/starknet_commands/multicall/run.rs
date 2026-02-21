use crate::Arguments;
use crate::starknet_commands::deploy::{ContractIdentifier, DeployArguments, DeployCommonArgs};
use crate::starknet_commands::invoke::{InvokeCommonArgs, execute_calls};
use crate::starknet_commands::multicall::ctx::MulticallCtx;
use crate::starknet_commands::multicall::deploy::MulticallDeploy;
use crate::starknet_commands::multicall::invoke::MulticallInvoke;
use anyhow::{Context, Result};
use camino::Utf8PathBuf;
use clap::Args;
use serde::Deserialize;
use sncast::WaitForTx;
use sncast::helpers::fee::FeeArgs;
use sncast::helpers::rpc::RpcArgs;
use sncast::response::errors::handle_starknet_command_error;
use sncast::response::multicall::run::MulticallRunResponse;
use sncast::response::ui::UI;
use starknet_rust::accounts::SingleOwnerAccount;
use starknet_rust::providers::JsonRpcClient;
use starknet_rust::providers::jsonrpc::HttpTransport;
use starknet_rust::signers::LocalWallet;
use starknet_types_core::felt::Felt;
use std::collections::HashMap;

#[derive(Args, Debug, Clone)]
#[command(about = "Execute a multicall from a .toml file", long_about = None)]
pub struct Run {
    /// Path to the toml file with declared operations
    #[arg(short = 'p', long = "path")]
    pub path: Utf8PathBuf,

    #[command(flatten)]
    pub fee_args: FeeArgs,

    #[command(flatten)]
    pub rpc: RpcArgs,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
enum Input {
    String(String),
    Number(i64),
}

fn inputs_to_calldata(inputs: Vec<Input>) -> Vec<String> {
    inputs
        .into_iter()
        .map(|input| match input {
            Input::String(s) => s,
            Input::Number(n) => n.to_string(),
        })
        .collect()
}

#[derive(Deserialize, Debug)]
struct DeployCall {
    class_hash: Felt,
    inputs: Vec<Input>,
    unique: bool,
    salt: Option<Felt>,
    id: String,
}

#[derive(Deserialize, Debug)]
struct InvokeCall {
    contract_address: String,
    function: String,
    inputs: Vec<Input>,
}

pub async fn run(
    run: Box<Run>,
    account: &SingleOwnerAccount<&JsonRpcClient<HttpTransport>, LocalWallet>,
    wait_config: WaitForTx,
    provider: &JsonRpcClient<HttpTransport>,
    ui: &UI,
) -> Result<MulticallRunResponse> {
    let fee_args = run.fee_args.clone();

    let contents = std::fs::read_to_string(&run.path)?;
    let items_map: HashMap<String, Vec<toml::Value>> =
        toml::from_str(&contents).with_context(|| format!("Failed to parse {}", run.path))?;

    let mut ctx = MulticallCtx::new(provider);
    let mut calls = vec![];

    for call in items_map.get("call").unwrap_or(&vec![]) {
        let call_type = call.get("call_type");
        if call_type.is_none() {
            anyhow::bail!("`Field call_type` is missing in a call specification");
        }

        match call_type.unwrap().as_str() {
            Some("deploy") => {
                let deploy_call: DeployCall = toml::from_str(toml::to_string(&call)?.as_str())
                    .context("Failed to parse toml `deploy` call")?;
                let arguments = DeployArguments {
                    constructor_calldata: Some(inputs_to_calldata(deploy_call.inputs.clone())),
                    arguments: None,
                };
                let id = if deploy_call.id.is_empty() {
                    None
                } else {
                    Some(deploy_call.id.clone())
                };

                let deploy = MulticallDeploy {
                    common: DeployCommonArgs {
                        contract_identifier: ContractIdentifier {
                            class_hash: Some(deploy_call.class_hash),
                            contract_name: None,
                        },
                        arguments,
                        salt: deploy_call.salt,
                        unique: deploy_call.unique,
                        package: None,
                    },
                    id,
                };

                let call = deploy.to_call(account, &mut ctx).await?;
                calls.push(call);
            }
            Some("invoke") => {
                let invoke_call: InvokeCall = toml::from_str(toml::to_string(&call)?.as_str())
                    .context("Failed to parse toml `invoke` call")?;

                let contract_address =
                    if let Some(address) = ctx.get_address_by_id(&invoke_call.contract_address) {
                        address
                    } else {
                        invoke_call
                            .contract_address
                            .parse()
                            .context("Failed to parse contract address in `invoke` call")?
                    };

                let arguments = Arguments {
                    calldata: Some(inputs_to_calldata(invoke_call.inputs.clone())),
                    arguments: None,
                };

                let invoke = MulticallInvoke {
                    common: InvokeCommonArgs {
                        contract_address: contract_address.to_string(),
                        function: invoke_call.function,
                        arguments,
                    },
                    id: None,
                };

                let call = invoke
                    .to_call(&mut ctx)
                    .await
                    .context("Failed to process `invoke` call")?;
                calls.push(call);
            }
            Some(unsupported) => {
                anyhow::bail!("Unsupported call type found = {unsupported}");
            }
            None => anyhow::bail!("Field `call_type` is missing in a call specification"),
        }
    }

    execute_calls(account, calls, fee_args, None, wait_config, ui)
        .await
        .map(Into::into)
        .map_err(handle_starknet_command_error)
}
