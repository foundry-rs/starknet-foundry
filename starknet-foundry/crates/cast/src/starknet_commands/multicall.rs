use crate::helpers::constants::DEFAULT_MULTICALL_CONTENTS;
use crate::starknet_commands::{
    deploy::{deploy, print_deploy_result},
    invoke::{invoke, print_invoke_result},
};
use anyhow::{bail, Result};
use clap::{Args, Subcommand};
use cast::parse_number;
use serde::Deserialize;
use starknet::accounts::SingleOwnerAccount;
use starknet::core::types::FieldElement;
use starknet::providers::jsonrpc::HttpTransport;
use starknet::providers::JsonRpcClient;
use starknet::signers::LocalWallet;
use std::collections::HashMap;
use camino::Utf8PathBuf;

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
struct DeployCall {
    call_type: String,
    class_hash: FieldElement,
    inputs: Vec<FieldElement>,
    max_fee: Option<FieldElement>,
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
    max_fee: Option<FieldElement>,
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
        path: Utf8PathBuf,
    },
    New {
        /// output path to the file where the template is going to be saved
        #[clap(short = 'p', long = "output-path")]
        output_path: Option<Utf8PathBuf>,

        /// if the file specified in output-path exists, this flag decides if it is going to be overwritten
        #[clap(short = 'o', long = "overwrite")]
        overwrite: Option<bool>,
    },
}

pub async fn run(
    path: &Utf8PathBuf,
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
                let result = deploy(
                    deploy_call.class_hash,
                    deploy_call.inputs,
                    deploy_call.salt,
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
                let mut contract_address = &invoke_call.contract_address;
                if let Some(addr) = contracts.get(&invoke_call.contract_address) {
                    contract_address = addr;
                }
                let result = invoke(
                    parse_number(contract_address)
                        .expect("Unable to parse contract address to FieldElement"),
                    &invoke_call.function,
                    invoke_call.inputs,
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

pub fn new(maybe_output_path: Option<Utf8PathBuf>, overwrite: bool) -> Result<()> {
    if let Some(output_path) = maybe_output_path {
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
        std::fs::write(output_path.clone(), DEFAULT_MULTICALL_CONTENTS)?;
        println!("Multicall template successfully saved in {output_path}");
    } else {
        println!("{DEFAULT_MULTICALL_CONTENTS}");
    }

    Ok(())
}
