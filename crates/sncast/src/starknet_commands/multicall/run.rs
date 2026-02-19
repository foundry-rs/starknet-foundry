use crate::Arguments;
use crate::starknet_commands::deploy::{ContractIdentifier, DeployCommonArgs};
use crate::starknet_commands::invoke::execute_calls;
use crate::starknet_commands::multicall::ctx::MulticallCtx;
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
use sncast::{WaitForTx, extract_or_generate_salt, get_contract_class, udc_uniqueness};
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

async fn process_deploy(deploy: &DeployCommonArgs, provider: &JsonRpcClient<HttpTransport>, ctx: &mut MulticallCtx, account: &SingleOwnerAccount<&JsonRpcClient<HttpTransport>, LocalWallet>) -> Result<()> {
    let salt = extract_or_generate_salt(deploy.salt);
    let class_hash = deploy
        .contract_identifier
        .class_hash
        .expect("Class hash must be provided for deploy calls");
    let contract_class = get_contract_class(class_hash, &provider).await?;
    let selector = get_selector_from_name("deployContract")?;

    let replaced_calldata = deploy.arguments.calldata.as_ref().map(|calldata| {
        calldata
            .iter()
            .map(|arg| {
                if let Some(addr) = ctx.get_address_by_id(arg) {
                    addr.to_string()
                } else {
                    arg.clone()
                }
            })
            .collect()
    });
    
    let constructor_calldata = deploy
        .arguments
        .try_into_calldata(contract_class, &selector)
        .expect("Failed to convert deploy arguments to calldata");
    let mut calldata = vec![
        class_hash,
        salt,
        Felt::from(u8::from(deploy.unique)),
        constructor_calldata.len().into(),
    ];

    calldata.extend(constructor_calldata);

    let contract_address = get_udc_deployed_address(
        salt,
        class_hash,
        &udc_uniqueness(deploy.unique, account.address()),
        &constructor_calldata,
    );

    // Store the contract address in the context with the provided id for later use in invoke calls
    if let Some(id) = &deploy.contract_identifier.contract_name {
        if ctx.get_address_by_id(id).is_some() {
            anyhow::bail!("Duplicate id found: {}", id);
        }
        ctx.insert_id_to_address(id.clone(), contract_address)?;
    }

    let call = Call {
        to: UDC_ADDRESS,
        selector,
        calldata,
    };
    ctx.add_call(call);

    Ok(())
    
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

    // let mut contracts = HashMap::new();
    // let mut parsed_calls: Vec<Call> = vec![];

    let mut ctx = MulticallCtx::new();

    for call in items_map.get("call").unwrap_or(&vec![]) {
        let call_type = call.get("call_type");
        if call_type.is_none() {
            anyhow::bail!("`Field call_type` is missing in a call specification");
        }

        match call_type.unwrap().as_str() {
            Some("deploy") => {
                let deploy_call: DeployCall = toml::from_str(toml::to_string(&call)?.as_str())
                    .context("Failed to parse toml `deploy` call")?;

                    
                let deploy = DeployCommonArgs {
                    contract_identifier: ContractIdentifier {
                        class_hash: Some(deploy_call.class_hash),
                        contract_name: None,
                    },
                    arguments: Arguments {
                        calldata: Some(
                            deploy_call
                                .inputs
                                .iter()
                                .map(|input| match input {
                                    Input::String(s) => s.clone(),
                                    Input::Number(n) => n.to_string(),
                                })
                                .collect(),
                        ),
                        arguments: None,
                    },
                    salt: deploy_call.salt,
                    unique: deploy_call.unique,
                };

                process_deploy(&deploy, 
                    &provider,
                    &mut ctx, 
                    account).await?;
            }
            Some("invoke") => {
                // let invoke_call: InvokeCall = toml::from_str(toml::to_string(&call)?.as_str())
                //     .context("Failed to parse toml `invoke` call")?;
                // let mut contract_address = &invoke_call.contract_address;
                // if let Some(addr) = contracts.get(&invoke_call.contract_address) {
                //     contract_address = addr;
                // }

                // let calldata = parse_inputs(&invoke_call.inputs, &contracts)?;

                // parsed_calls.push(Call {
                //     to: contract_address
                //         .parse()
                //         .context("Failed to parse contract address to Felt")?,
                //     selector: get_selector_from_name(&invoke_call.function)?,
                //     calldata,
                // });
            }
            Some(unsupported) => {
                anyhow::bail!("Unsupported call type found = {unsupported}");
            }
            None => anyhow::bail!("Field `call_type` is missing in a call specification"),
        }
    }

    execute_calls(account, ctx.calls().to_vec(), fee_args, None, wait_config, ui)
        .await
        .map(Into::into)
        .map_err(handle_starknet_command_error)
}

// fn parse_inputs(inputs: &Vec<Input>, contracts: &HashMap<String, String>) -> Result<Vec<Felt>> {
//     let mut parsed_inputs = Vec::new();
//     for input in inputs {
//         let felt_value = match input {
//             Input::String(s) => {
//                 let resolved = contracts.get(s).unwrap_or(s);
//                 resolved
//                     .parse()
//                     .context(format!("Failed to parse input '{resolved}' to Felt"))?
//             }
//             Input::Number(n) => (*n).into(),
//         };
//         parsed_inputs.push(felt_value);
//     }

//     Ok(parsed_inputs)
// }
