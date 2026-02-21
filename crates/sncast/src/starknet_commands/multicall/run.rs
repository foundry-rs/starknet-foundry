use crate::starknet_commands::invoke::execute_calls;
use crate::starknet_commands::multicall::contracts_registry::ContractsRegistry;
use anyhow::{Context, Result};
use camino::Utf8PathBuf;
use clap::Args;
use serde::Deserialize;
use sncast::helpers::constants::UDC_ADDRESS;
use sncast::helpers::fee::FeeArgs;
use sncast::helpers::rpc::RpcArgs;
use sncast::response::errors::handle_starknet_command_error;
use sncast::response::multicall::run::MulticallRunResponse;
use sncast::response::ui::UI;
use sncast::{WaitForTx, extract_or_generate_salt, udc_uniqueness};
use starknet_rust::accounts::{Account, SingleOwnerAccount};
use starknet_rust::core::types::Call;
use starknet_rust::core::utils::{get_selector_from_name, get_udc_deployed_address};
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
    ui: &UI,
) -> Result<MulticallRunResponse> {
    let fee_args = run.fee_args.clone();

    let contents = std::fs::read_to_string(&run.path)?;
    let items_map: HashMap<String, Vec<toml::Value>> =
        toml::from_str(&contents).with_context(|| format!("Failed to parse {}", run.path))?;

    let mut contracts_registry = ContractsRegistry::new();
    let mut parsed_calls: Vec<Call> = vec![];

    for call in items_map.get("call").unwrap_or(&vec![]) {
        let call_type = call.get("call_type");
        if call_type.is_none() {
            anyhow::bail!("`Field call_type` is missing in a call specification");
        }

        match call_type.unwrap().as_str() {
            Some("deploy") => {
                let deploy_call: DeployCall = toml::from_str(toml::to_string(&call)?.as_str())
                    .context("Failed to parse toml `deploy` call")?;

                let salt = extract_or_generate_salt(deploy_call.salt);
                let mut calldata = vec![
                    deploy_call.class_hash,
                    salt,
                    Felt::from(u8::from(deploy_call.unique)),
                    deploy_call.inputs.len().into(),
                ];

                let parsed_inputs = parse_inputs(&deploy_call.inputs, &contracts_registry)?;
                calldata.extend(&parsed_inputs);

                parsed_calls.push(Call {
                    to: UDC_ADDRESS,
                    selector: get_selector_from_name("deployContract")?,
                    calldata,
                });

                let contract_address = get_udc_deployed_address(
                    salt,
                    deploy_call.class_hash,
                    &udc_uniqueness(deploy_call.unique, account.address()),
                    &parsed_inputs,
                );
                contracts_registry.insert_new_id_to_address(deploy_call.id, contract_address)?;
            }
            Some("invoke") => {
                let invoke_call: InvokeCall = toml::from_str(toml::to_string(&call)?.as_str())
                    .context("Failed to parse toml `invoke` call")?;
                let contract_address = contracts_registry
                    .get_address_by_id(&invoke_call.contract_address)
                    .unwrap_or_else(|| {
                        invoke_call
                            .contract_address
                            .parse()
                            .expect("Failed to parse contract address to Felt")
                    });
                let calldata = parse_inputs(&invoke_call.inputs, &contracts_registry)?;

                parsed_calls.push(Call {
                    to: contract_address,
                    selector: get_selector_from_name(&invoke_call.function)?,
                    calldata,
                });
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

fn parse_inputs(inputs: &Vec<Input>, contracts_registry: &ContractsRegistry) -> Result<Vec<Felt>> {
    let mut parsed_inputs = Vec::new();
    for input in inputs {
        let felt_value = match input {
            Input::String(s) => {
                let resolved_address = contracts_registry.get_address_by_id(s);
                resolved_address.unwrap_or(s.parse()?)
            }
            Input::Number(n) => (*n).into(),
        };
        parsed_inputs.push(felt_value);
    }

    Ok(parsed_inputs)
}
