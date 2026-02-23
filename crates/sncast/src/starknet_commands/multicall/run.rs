use crate::Arguments;
use crate::starknet_commands::deploy::{ContractIdentifier, DeployArguments, DeployCommonArgs};
use crate::starknet_commands::invoke::{InvokeCommonArgs, execute_calls};
use crate::starknet_commands::multicall::contracts_registry::ContractsRegistry;
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
use starknet_rust::core::types::Call;
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

#[derive(Deserialize, Debug)]
#[serde(untagged)]
enum Input {
    String(String),
    Number(i64),
}

#[derive(Deserialize, Debug)]
pub(crate) struct DeployItem {
    class_hash: Felt,
    inputs: Vec<Input>,
    unique: bool,
    salt: Option<Felt>,
    id: String,
}

#[derive(Deserialize, Debug)]
pub(crate) struct InvokeItem {
    contract_address: String,
    function: String,
    inputs: Vec<Input>,
}

pub async fn run(
    run: Box<Run>,
    account: &SingleOwnerAccount<&JsonRpcClient<HttpTransport>, LocalWallet>,
    provider: &JsonRpcClient<HttpTransport>,
    wait_config: WaitForTx,
    ui: &UI,
) -> Result<MulticallRunResponse> {
    let fee_args = run.fee_args.clone();

    let contents = std::fs::read_to_string(&run.path)?;
    let items_map: HashMap<String, Vec<toml::Value>> =
        toml::from_str(&contents).with_context(|| format!("Failed to parse {}", run.path))?;

    let mut contracts_registry = ContractsRegistry::new(provider);
    let mut parsed_calls: Vec<Call> = vec![];

    for call in items_map.get("call").unwrap_or(&vec![]) {
        let call_type = call.get("call_type");
        if call_type.is_none() {
            anyhow::bail!("`Field call_type` is missing in a call specification");
        }

        match call_type.unwrap().as_str() {
            Some("deploy") => {
                let item: DeployItem = toml::from_str(toml::to_string(&call)?.as_str())
                    .context("Failed to parse toml `deploy` call")?;
                let constructor_calldata = parse_inputs(&item.inputs, &contracts_registry)?;
                let deploy = MulticallDeploy {
                    common: DeployCommonArgs {
                        contract_identifier: ContractIdentifier {
                            class_hash: Some(item.class_hash),
                            contract_name: None,
                        },
                        arguments: DeployArguments {
                            constructor_calldata: Some(constructor_calldata),
                            arguments: None,
                        },
                        salt: item.salt,
                        unique: item.unique,
                        package: None,
                    },
                    id: if item.id.is_empty() {
                        None
                    } else {
                        Some(item.id)
                    },
                };

                let call = deploy.build_call(account, &mut contracts_registry).await?;
                parsed_calls.push(call);
            }
            Some("invoke") => {
                let item: InvokeItem = toml::from_str(toml::to_string(&call)?.as_str())
                    .context("Failed to parse toml `invoke` call")?;
                let calldata = parse_inputs(&item.inputs, &contracts_registry)?;
                let contract_address = if let Some(addr) =
                    contracts_registry.get_address_by_id(&item.contract_address)
                {
                    addr
                } else {
                    item.contract_address.parse()?
                };
                let invoke = MulticallInvoke {
                    common: InvokeCommonArgs {
                        contract_address: contract_address.to_string(),
                        function: item.function,
                        arguments: Arguments {
                            calldata: Some(calldata),
                            arguments: None,
                        },
                    },
                };

                let call = invoke.build_call(&mut contracts_registry).await?;
                parsed_calls.push(call);
            }
            Some(unsupported) => {
                anyhow::bail!("Unsupported call type found = {unsupported}");
            }
            None => anyhow::bail!("Field `call_type` is missing in a call specification"),
        }
    }

    execute_calls(account, parsed_calls, fee_args, None, wait_config, ui)
        .await
        .map(Into::into)
        .map_err(handle_starknet_command_error)
}

fn parse_inputs(
    inputs: &Vec<Input>,
    contracts_registry: &ContractsRegistry,
) -> Result<Vec<String>> {
    let mut parsed_inputs = Vec::new();
    for input in inputs {
        let felt_value = match input {
            Input::String(s) => {
                let resolved_address = contracts_registry.get_address_by_id(s);
                if let Some(address) = resolved_address {
                    address.to_string()
                } else {
                    let felt: Felt = s.parse()?;
                    felt.to_string()
                }
            }
            Input::Number(n) => n.to_string(),
        };
        parsed_inputs.push(felt_value);
    }

    Ok(parsed_inputs)
}
