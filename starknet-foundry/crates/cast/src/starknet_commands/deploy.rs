use anyhow::{anyhow, Result};
use clap::Args;
use rand::rngs::OsRng;
use rand::RngCore;

use crate::helpers::constants::UDC_ADDRESS;
use cast::print_formatted;
use cast::{handle_rpc_error, handle_wait_for_tx_result};
use starknet::accounts::AccountError::Provider;
use starknet::accounts::{Account, ConnectedAccount, SingleOwnerAccount};
use starknet::contract::ContractFactory;
use starknet::core::types::FieldElement;
use starknet::core::utils::UdcUniqueness::{NotUnique, Unique};
use starknet::core::utils::{get_udc_deployed_address, UdcUniqueSettings};
use starknet::providers::jsonrpc::HttpTransport;
use starknet::providers::JsonRpcClient;
use starknet::signers::LocalWallet;

#[derive(Args)]
#[command(about = "Deploy a contract on Starknet")]
pub struct Deploy {
    /// Class hash of contract to deploy
    #[clap(short = 'g', long)]
    pub class_hash: FieldElement,

    /// Calldata for the contract constructor
    #[clap(short, long, value_delimiter = ' ')]
    pub constructor_calldata: Vec<FieldElement>,

    /// Salt for the address
    #[clap(short, long)]
    pub salt: Option<FieldElement>,

    /// If true, salt will be modified with an account address
    #[clap(short, long)]
    pub unique: bool,

    /// Max fee for the transaction. If not provided, max fee will be automatically estimated
    #[clap(short, long)]
    pub max_fee: Option<FieldElement>,
}

pub fn print_deploy_result(
    deploy_result: Result<(FieldElement, FieldElement)>,
    int_format: bool,
    json: bool,
) -> Result<()> {
    match deploy_result {
        Ok((transaction_hash, contract_address)) => print_formatted(
            vec![
                ("command", "Deploy".to_string()),
                ("contract_address", format!("{contract_address}")),
                ("transaction_hash", format!("{transaction_hash}")),
            ],
            int_format,
            json,
            false,
        )?,
        Err(error) => {
            print_formatted(vec![("error", error.to_string())], int_format, json, true)?;
        }
    };
    Ok(())
}

pub async fn deploy(
    class_hash: FieldElement,
    constructor_calldata: Vec<FieldElement>,
    salt: Option<FieldElement>,
    unique: bool,
    max_fee: Option<FieldElement>,
    account: &SingleOwnerAccount<&JsonRpcClient<HttpTransport>, LocalWallet>,
) -> Result<(FieldElement, FieldElement)> {
    let salt = match salt {
        Some(salt) => salt,
        None => FieldElement::from(OsRng.next_u64()),
    };

    let factory = ContractFactory::new(class_hash, account);
    let deployment = factory.deploy(&constructor_calldata, salt, unique);

    let execution = if let Some(max_fee) = max_fee {
        deployment.max_fee(max_fee)
    } else {
        deployment
    };

    let result = execution.send().await;

    match result {
        Ok(result) => {
            handle_wait_for_tx_result(
                account.provider(),
                result.transaction_hash,
                (
                    result.transaction_hash,
                    get_udc_deployed_address(
                        salt,
                        class_hash,
                        &if unique {
                            Unique(UdcUniqueSettings {
                                deployer_address: account.address(),
                                udc_contract_address: FieldElement::from_hex_be(UDC_ADDRESS)?,
                            })
                        } else {
                            NotUnique
                        },
                        &constructor_calldata,
                    ),
                ),
            )
            .await
        }
        Err(Provider(error)) => handle_rpc_error(error),
        _ => Err(anyhow!("Unknown RPC error")),
    }
}
