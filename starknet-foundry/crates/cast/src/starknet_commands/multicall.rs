use crate::starknet_commands::{
    deploy::{deploy, print_deploy_result},
    invoke::{invoke, print_invoke_result},
};
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
    inputs: Vec<String>,
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
    inputs: Vec<String>,
    max_fee: Option<u128>,
}

#[derive(Args)]
#[command(about = "Execute multiple calls at once", long_about = None)]
pub struct Multicall {
    #[clap(short = 'p', long = "path")]
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
    let empty_vec = &vec![];
    let calls = items_map.get("call").unwrap_or(empty_vec);
    for call in calls {
        let call_type = call.get("call_type");
        if call_type.is_none() {
            anyhow::bail!("`call_type` field is missing in a call specification");
        }
        match call_type.unwrap().as_str() {
            Some("\"deploy\"") => {
                let deploy_call: DeployCall = toml::from_str(call.to_string().as_str())
                    .expect("failed to parse toml `deploy` call");
                let inputs_as_strings_slices: Vec<&str> =
                    deploy_call.inputs.iter().map(|s| s.as_str()).collect();
                let result = deploy(
                    &deploy_call.class_hash,
                    inputs_as_strings_slices,
                    deploy_call.salt.as_deref(),
                    deploy_call.unique,
                    deploy_call.max_fee,
                    account,
                )
                .await;
                print_deploy_result(result, int_format, json).await?;
            }
            Some("\"invoke\"") => {
                let invoke_call: InvokeCall = toml::from_str(call.to_string().as_str())
                    .expect("failed to parse toml `invoke` call");
                let inputs_as_strings_slices: Vec<&str> =
                    invoke_call.inputs.iter().map(|s| s.as_str()).collect();
                let result = invoke(
                    &invoke_call.contract_address,
                    &invoke_call.function,
                    inputs_as_strings_slices,
                    invoke_call.max_fee,
                    account,
                )
                .await;
                print_invoke_result(result, int_format, json).await?;
            }
            Some(unsupported) => {
                anyhow::bail!("unsupported call type found: {}", unsupported);
            }
            None => anyhow::bail!("`call_type` field is missing in a call specification"),
        }
    }

    Ok(())
}
