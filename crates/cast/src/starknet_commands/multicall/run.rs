use crate::starknet_commands::invoke::execute_calls;
use anyhow::{anyhow, Context, Result};
use camino::Utf8PathBuf;
use cast::helpers::constants::UDC_ADDRESS;
use cast::helpers::response_structs::InvokeResponse;
use cast::{extract_or_generate_salt, parse_number, udc_uniqueness};
use clap::Args;
use serde::Deserialize;
use starknet::accounts::{Account, Call, SingleOwnerAccount};
use starknet::core::types::FieldElement;
use starknet::core::utils::{get_selector_from_name, get_udc_deployed_address};
use starknet::providers::jsonrpc::HttpTransport;
use starknet::providers::JsonRpcClient;
use starknet::signers::LocalWallet;
use std::collections::HashMap;

#[derive(Args, Debug)]
#[command(about = "Execute a multicall from a .toml file", long_about = None)]
pub struct Run {
    /// Path to the toml file with declared operations
    #[clap(short = 'p', long = "path")]
    pub path: Utf8PathBuf,

    /// Max fee for the transaction. If not provided, max fee will be automatically estimated
    #[clap(short, long)]
    pub max_fee: Option<FieldElement>,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
struct DeployCall {
    call_type: String,
    class_hash: FieldElement,
    inputs: Vec<FieldElement>,
    unique: bool,
    salt: Option<FieldElement>,
    id: String,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
struct InvokeCall {
    call_type: String,
    contract_address: String,
    function: String,
    inputs: Vec<FieldElement>,
}

pub async fn run(
    path: &Utf8PathBuf,
    account: &mut SingleOwnerAccount<&JsonRpcClient<HttpTransport>, LocalWallet>,
    max_fee: Option<FieldElement>,
    wait: bool,
) -> Result<InvokeResponse> {
    let contents = std::fs::read_to_string(path)?;
    let items_map: HashMap<String, Vec<toml::Value>> =
        toml::from_str(&contents).map_err(|_| anyhow!("Failed to parse {path}"))?;

    let mut contracts = HashMap::new();
    let mut parsed_calls: Vec<Call> = vec![];

    for call in items_map.get("call").unwrap_or(&vec![]) {
        let call_type = call.get("call_type");
        if call_type.is_none() {
            anyhow::bail!("`call_type` field is missing in a call specification");
        }

        match call_type.unwrap().as_str() {
            Some("deploy") => {
                let deploy_call: DeployCall = toml::from_str(toml::to_string(&call)?.as_str())
                    .map_err(|_| anyhow!("Failed to parse toml `deploy` call"))?;

                let salt = extract_or_generate_salt(deploy_call.salt);
                let mut calldata = vec![
                    deploy_call.class_hash,
                    salt,
                    FieldElement::from(u8::from(deploy_call.unique)),
                    deploy_call.inputs.len().into(),
                ];
                deploy_call
                    .inputs
                    .iter()
                    .for_each(|item| calldata.push(*item));

                parsed_calls.push(Call {
                    to: parse_number(UDC_ADDRESS)?,
                    selector: get_selector_from_name("deployContract")?,
                    calldata,
                });

                let contract_address = get_udc_deployed_address(
                    salt,
                    deploy_call.class_hash,
                    &udc_uniqueness(deploy_call.unique, account.address()),
                    &deploy_call.inputs,
                );
                contracts.insert(deploy_call.id, contract_address.to_string());
            }
            Some("invoke") => {
                let invoke_call: InvokeCall = toml::from_str(toml::to_string(&call)?.as_str())
                    .context("failed to parse toml `invoke` call")?;
                let mut contract_address = &invoke_call.contract_address;
                if let Some(addr) = contracts.get(&invoke_call.contract_address) {
                    contract_address = addr;
                }

                parsed_calls.push(Call {
                    to: parse_number(contract_address)
                        .context("Unable to parse contract address to FieldElement")?,
                    selector: get_selector_from_name(&invoke_call.function)?,
                    calldata: invoke_call.inputs,
                });
            }
            Some(unsupported) => {
                anyhow::bail!("unsupported call type found: {}", unsupported);
            }
            None => anyhow::bail!("`call_type` field is missing in a call specification"),
        }
    }

    execute_calls(account, parsed_calls, max_fee, wait).await
}
