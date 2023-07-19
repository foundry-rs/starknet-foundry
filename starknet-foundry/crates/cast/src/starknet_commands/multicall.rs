use crate::helpers::constants::DEFAULT_MULTICALL_CONTENTS;
use crate::starknet_commands::{
    deploy::{deploy, print_deploy_result},
    invoke::{invoke, print_invoke_result},
};
use anyhow::{bail, Result};
use clap::{Args, Subcommand};
use serde::Deserialize;
use starknet::accounts::SingleOwnerAccount;
use starknet::providers::jsonrpc::HttpTransport;
use starknet::providers::JsonRpcClient;
use starknet::signers::LocalWallet;
use std::collections::HashMap;
use std::path::Path;

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
struct DeployCall {
    call_type: String,
    class_hash: String,
    inputs: Vec<String>,
    max_fee: Option<u128>,
    unique: bool,
    salt: Option<String>,
    id: String,
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
    #[clap(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    Run {
        /// path to the toml file with declared operations
        #[clap(short = 'p', long = "path")]
        path: String,
    },
    New {
        /// output path to the file where the template is going to be saved
        #[clap(short = 'p', long = "output-path")]
        output_path: Option<String>,

        /// if the file specified in output-path exists, this flag decides if it is going to be overwritten
        #[clap(short = 'o', long = "overwrite")]
        overwrite: Option<bool>,
    },
}

pub async fn run(
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
    let mut contracts = HashMap::new();
    for call in calls {
        let call_type = call.get("call_type");
        if call_type.is_none() {
            anyhow::bail!("`call_type` field is missing in a call specification");
        }
        match call_type.unwrap().as_str() {
            Some("deploy") => {
                let deploy_call: DeployCall = toml::from_str(call.to_string().as_str())
                    .expect("failed to parse toml `deploy` call");
                let inputs_as_strings_slices: Vec<&str> = deploy_call
                    .inputs
                    .iter()
                    .map(std::string::String::as_str)
                    .collect();
                let result = deploy(
                    &deploy_call.class_hash,
                    inputs_as_strings_slices,
                    deploy_call.salt.as_deref(),
                    deploy_call.unique,
                    deploy_call.max_fee,
                    account,
                )
                .await;
                if let Ok((_, contract_address)) = result {
                    contracts.insert(deploy_call.id, contract_address.to_string());
                }
                print_deploy_result(result, int_format, json)?;
            }
            Some("invoke") => {
                let invoke_call: InvokeCall = toml::from_str(call.to_string().as_str())
                    .expect("failed to parse toml `invoke` call");
                let inputs_as_strings_slices: Vec<&str> = invoke_call
                    .inputs
                    .iter()
                    .map(std::string::String::as_str)
                    .collect();
                let mut contract_address = &invoke_call.contract_address;
                if let Some(addr) = contracts.get(&invoke_call.contract_address) {
                    contract_address = addr;
                }
                let result = invoke(
                    contract_address,
                    &invoke_call.function,
                    inputs_as_strings_slices,
                    invoke_call.max_fee,
                    account,
                )
                .await;
                print_invoke_result(result, int_format, json)?;
            }
            Some(unsupported) => {
                anyhow::bail!("unsupported call type found: {}", unsupported);
            }
            None => anyhow::bail!("`call_type` field is missing in a call specification"),
        }
    }

    Ok(())
}

pub fn new(maybe_output_path: Option<String>, overwrite: bool) -> Result<()> {
    if let Some(output_path) = maybe_output_path {
        let output_path = Path::new(output_path.as_str());
        if output_path.exists() {
            if !output_path.is_file() {
                bail!("output file cannot be a directory");
            }
            if !overwrite {
                bail!(
                    "output file already exists, if you want to overwrite it, use the `overwrite` flag"
                );
            }
        }
        std::fs::write(output_path, DEFAULT_MULTICALL_CONTENTS)?;
        println!(
            "Multicall template successfully saved in {}",
            output_path
                .to_str()
                .expect("failed to convert path to string")
        );
    } else {
        println!("{DEFAULT_MULTICALL_CONTENTS}");
    }

    Ok(())
}
