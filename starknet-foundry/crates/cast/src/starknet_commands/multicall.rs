use crate::starknet_commands::{deploy::deploy_and_print, invoke::invoke_and_print};
use anyhow::Result;
use clap::Args;
use serde::Deserialize;
use starknet::accounts::SingleOwnerAccount;
use starknet::providers::jsonrpc::HttpTransport;
use starknet::providers::JsonRpcClient;
use starknet::signers::LocalWallet;
use std::collections::HashMap;

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
struct DeployCall {
    call_type: String,
    class_hash: String,
    inputs: Vec<u32>,
    max_fee: Option<u128>,
    unique: bool,
    salt: Option<String>,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
struct InvokeCall {
    call_type: String,
    contract_address: String,
    function: String,
    inputs: Vec<u32>,
    max_fee: Option<u128>,
}

#[derive(Args)]
#[command(about = "Execute multiple calls at once", long_about = None)]
pub struct Multicall {
    /// Path to a .toml file containing the multi call specification
    pub path: String,
}

pub async fn multicall(
    path: &str,
    account: &mut SingleOwnerAccount<&JsonRpcClient<HttpTransport>, LocalWallet>,
    int_format: bool,
    json: bool,
) -> Result<()> {
    let contents = std::fs::read_to_string(path)?;
    let items_map: HashMap<String, Vec<toml::Value>> =
        toml::from_str(&contents).expect("failed to parse toml file");
    let calls = &items_map["call"];
    for call in calls {
        if let Some(call_type) = call.get("call_type") {
            match call_type.to_string().as_str() {
                "\"deploy\"" => {
                    let deploy_call: DeployCall = toml::from_str(call.to_string().as_str())
                        .expect("failed to parse toml `deploy` call");
                    let inputs_as_strings: Vec<String> = deploy_call
                        .inputs
                        .iter()
                        .map(|item| item.to_string())
                        .collect();
                    let inputs_as_strings_slices: Vec<&str> =
                        inputs_as_strings.iter().map(String::as_str).collect();
                    deploy_and_print(
                        deploy_call.class_hash.as_str(),
                        inputs_as_strings_slices,
                        deploy_call.salt.as_ref().map(|x| &**x),
                        deploy_call.unique,
                        deploy_call.max_fee,
                        account,
                        int_format,
                        json,
                    )
                    .await?;
                }
                "\"invoke\"" => {
                    let invoke_call: InvokeCall = toml::from_str(call.to_string().as_str())
                        .expect("failed to parse toml `invoke` call");
                    let inputs_as_strings: Vec<String> = invoke_call
                        .inputs
                        .iter()
                        .map(|item| item.to_string())
                        .collect();
                    let inputs_as_strings_slices: Vec<&str> =
                        inputs_as_strings.iter().map(String::as_str).collect();
                    invoke_and_print(
                        &invoke_call.contract_address[..],
                        &invoke_call.function,
                        inputs_as_strings_slices,
                        invoke_call.max_fee,
                        account,
                        int_format,
                        json,
                    )
                    .await?;
                }
                unsupported => {
                    anyhow::bail!("unsupported call type found: {}", unsupported);
                }
            }
        } else {
            anyhow::bail!("`call_type` field is missing in a call specification");
        }
    }

    Ok(())
}
