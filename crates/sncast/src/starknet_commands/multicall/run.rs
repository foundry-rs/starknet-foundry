use crate::starknet_commands::invoke::{execute_calls, InvokeVersion};
use anyhow::anyhow;
use anyhow::{Context, Result};
use camino::Utf8PathBuf;
use clap::Args;
use serde::Deserialize;
use sncast::helpers::constants::UDC_ADDRESS;
use sncast::helpers::error::token_not_supported_for_invoke;
use sncast::helpers::fee::{FeeArgs, FeeToken, PayableTransaction};
use sncast::helpers::rpc::RpcArgs;
use sncast::response::errors::handle_starknet_command_error;
use sncast::response::structs::InvokeResponse;
use sncast::{extract_or_generate_salt, impl_payable_transaction, udc_uniqueness, WaitForTx};
use starknet::accounts::{Account, SingleOwnerAccount};
use starknet::core::types::Call;
use starknet::core::utils::{get_selector_from_name, get_udc_deployed_address};
use starknet::providers::jsonrpc::HttpTransport;
use starknet::providers::JsonRpcClient;
use starknet::signers::LocalWallet;
use starknet_types_core::felt::Felt;
use std::collections::HashMap;

#[derive(Args, Debug, Clone)]
#[command(about = "Execute a multicall from a .toml file", long_about = None)]
pub struct Run {
    /// Path to the toml file with declared operations
    #[clap(short = 'p', long = "path")]
    pub path: Utf8PathBuf,

    #[clap(flatten)]
    pub fee_args: FeeArgs,

    /// Version of invoke (can be inferred from fee token)
    #[clap(short, long)]
    pub version: Option<InvokeVersion>,

    #[clap(flatten)]
    pub rpc: RpcArgs,
}

impl_payable_transaction!(Run, token_not_supported_for_invoke,
    InvokeVersion::V1 => FeeToken::Eth,
    InvokeVersion::V3 => FeeToken::Strk
);

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
    run: Run,
    account: &SingleOwnerAccount<&JsonRpcClient<HttpTransport>, LocalWallet>,
    wait_config: WaitForTx,
) -> Result<InvokeResponse> {
    let fee_token = run.validate_and_get_token()?;

    let fee_args = run.fee_args.clone().fee_token(fee_token);

    let contents = std::fs::read_to_string(&run.path)?;
    let items_map: HashMap<String, Vec<toml::Value>> =
        toml::from_str(&contents).with_context(|| format!("Failed to parse {}", run.path))?;

    let mut contracts = HashMap::new();
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

                let parsed_inputs = parse_inputs(&deploy_call.inputs, &contracts)?;
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
                contracts.insert(deploy_call.id, contract_address.to_string());
            }
            Some("invoke") => {
                let invoke_call: InvokeCall = toml::from_str(toml::to_string(&call)?.as_str())
                    .context("Failed to parse toml `invoke` call")?;
                let mut contract_address = &invoke_call.contract_address;
                if let Some(addr) = contracts.get(&invoke_call.contract_address) {
                    contract_address = addr;
                }

                let calldata = parse_inputs(&invoke_call.inputs, &contracts)?;

                parsed_calls.push(Call {
                    to: contract_address
                        .parse()
                        .context("Failed to parse contract address to Felt")?,
                    selector: get_selector_from_name(&invoke_call.function)?,
                    calldata,
                });
            }
            Some(unsupported) => {
                anyhow::bail!("Unsupported call type found = {}", unsupported);
            }
            None => anyhow::bail!("Field `call_type` is missing in a call specification"),
        }
    }

    execute_calls(account, parsed_calls, fee_args, None, wait_config)
        .await
        .map_err(handle_starknet_command_error)
}

fn parse_inputs(inputs: &Vec<Input>, contracts: &HashMap<String, String>) -> Result<Vec<Felt>> {
    let mut parsed_inputs = Vec::new();
    for input in inputs {
        let felt_value = match input {
            Input::String(s) => {
                let resolved = contracts.get(s).unwrap_or(s);
                resolved
                    .parse()
                    .context(format!("Failed to parse input '{resolved}' to Felt"))?
            }
            Input::Number(n) => (*n).into(),
        };
        parsed_inputs.push(felt_value);
    }

    Ok(parsed_inputs)
}
