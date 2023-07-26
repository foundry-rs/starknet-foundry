use anyhow::{anyhow, Result};
use camino::Utf8PathBuf;
use cast::helpers::constants::UDC_ADDRESS;
use cast::{handle_rpc_error, handle_wait_for_tx_result, parse_number};
use clap::Args;
use rand::rngs::OsRng;
use rand::RngCore;
use serde::Deserialize;
use starknet::accounts::AccountError::Provider;
use starknet::accounts::{Account, Call, ConnectedAccount, SingleOwnerAccount};
use starknet::core::types::FieldElement;
use starknet::core::utils::UdcUniqueness::{NotUnique, Unique};
use starknet::core::utils::{get_selector_from_name, get_udc_deployed_address, UdcUniqueSettings};
use starknet::providers::jsonrpc::HttpTransport;
use starknet::providers::JsonRpcClient;
use starknet::signers::LocalWallet;
use std::collections::HashMap;

#[derive(Args, Debug)]
#[command(about = "Declare a contract to starknet", long_about = None)]
pub struct Run {
    /// path to the toml file with declared operations
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
) -> Result<FieldElement> {
    let contents = std::fs::read_to_string(path)?;
    let items_map: HashMap<String, Vec<toml::Value>> =
        toml::from_str(&contents).expect("failed to parse toml file");

    let mut contracts = HashMap::new();
    let mut parsed_calls: Vec<Call> = vec![];

    for call in items_map.get("call").unwrap_or(&vec![]) {
        let call_type = call.get("call_type");
        if call_type.is_none() {
            anyhow::bail!("`call_type` field is missing in a call specification");
        }

        match call_type.unwrap().as_str() {
            Some("deploy") => {
                let deploy_call: DeployCall = toml::from_str(call.to_string().as_str())
                    .expect("failed to parse toml `deploy` call");

                let salt = deploy_call
                    .salt
                    .unwrap_or(FieldElement::from(OsRng.next_u64()));
                let mut calldata = vec![
                    deploy_call.class_hash,
                    salt,
                    if deploy_call.unique {
                        FieldElement::ONE
                    } else {
                        FieldElement::ZERO
                    },
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
                    &if deploy_call.unique {
                        Unique(UdcUniqueSettings {
                            deployer_address: account.address(),
                            udc_contract_address: FieldElement::from_hex_be(UDC_ADDRESS)?,
                        })
                    } else {
                        NotUnique
                    },
                    &deploy_call.inputs,
                );
                contracts.insert(deploy_call.id, contract_address.to_string());
            }
            Some("invoke") => {
                let invoke_call: InvokeCall = toml::from_str(call.to_string().as_str())
                    .expect("failed to parse toml `invoke` call");
                let mut contract_address = &invoke_call.contract_address;
                if let Some(addr) = contracts.get(&invoke_call.contract_address) {
                    contract_address = addr;
                }

                parsed_calls.push(Call {
                    to: parse_number(contract_address)
                        .expect("Unable to parse contract address to FieldElement"),
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

    let execution = account.execute(parsed_calls);
    let execution = if let Some(max_fee) = max_fee {
        execution.max_fee(max_fee)
    } else {
        execution
    };

    match execution.send().await {
        Ok(result) => {
            handle_wait_for_tx_result(
                account.provider(),
                result.transaction_hash,
                result.transaction_hash,
            )
            .await
        }
        Err(Provider(error)) => handle_rpc_error(error),
        _ => Err(anyhow!("Unknown RPC error")),
    }
}
