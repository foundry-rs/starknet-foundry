use anyhow::{anyhow, Result};
use clap::Args;
use starknet::accounts::AccountError::Provider;
use starknet::accounts::{Account, ConnectedAccount, SingleOwnerAccount};
use starknet::contract::ContractFactory;
use starknet::core::types::FieldElement;
use starknet::core::utils::get_udc_deployed_address;
use starknet::providers::jsonrpc::HttpTransport;
use starknet::providers::JsonRpcClient;
use starknet::signers::LocalWallet;

use cast::helpers::response_structs::DeployResponse;
use cast::{extract_or_generate_salt, udc_uniqueness};
use cast::{handle_rpc_error, handle_wait_for_tx};

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

pub async fn deploy(
    class_hash: FieldElement,
    constructor_calldata: Vec<FieldElement>,
    salt: Option<FieldElement>,
    unique: bool,
    max_fee: Option<FieldElement>,
    account: &SingleOwnerAccount<&JsonRpcClient<HttpTransport>, LocalWallet>,
    wait: bool,
) -> Result<DeployResponse> {
    let salt = extract_or_generate_salt(salt);

    let factory = ContractFactory::new(class_hash, account);
    let deployment = factory.deploy(constructor_calldata.clone(), salt, unique);

    let execution = if let Some(max_fee) = max_fee {
        deployment.max_fee(max_fee)
    } else {
        deployment
    };

    let result = execution.send().await;

    match result {
        Ok(result) => {
            handle_wait_for_tx(
                account.provider(),
                result.transaction_hash,
                DeployResponse {
                    contract_address: get_udc_deployed_address(
                        salt,
                        class_hash,
                        &udc_uniqueness(unique, account.address()),
                        &constructor_calldata,
                    ),
                    transaction_hash: result.transaction_hash,
                },
                wait,
            )
            .await
        }
        Err(Provider(error)) => handle_rpc_error(error),
        _ => Err(anyhow!("Unknown RPC error")),
    }
}
