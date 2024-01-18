use anyhow::{anyhow, Result};
use clap::Args;
use sncast::response::structs::{DeployResponse, Hex};
use starknet::accounts::AccountError::Provider;
use starknet::accounts::{Account, ConnectedAccount, SingleOwnerAccount};
use starknet::contract::ContractFactory;
use starknet::core::types::FieldElement;
use starknet::core::utils::get_udc_deployed_address;
use starknet::providers::jsonrpc::HttpTransport;
use starknet::providers::JsonRpcClient;
use starknet::signers::LocalWallet;

use sncast::{extract_or_generate_salt, udc_uniqueness};
use sncast::{handle_rpc_error, handle_wait_for_tx, WaitForTx};

#[derive(Args)]
#[command(about = "Deploy a contract on Starknet")]
pub struct Deploy {
    /// Class hash of contract to deploy
    #[clap(short = 'g', long)]
    pub class_hash: FieldElement,

    /// Calldata for the contract constructor
    #[clap(short, long, value_delimiter = ' ', num_args = 1..)]
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

    /// Nonce of the transaction. If not provided, nonce will be set automatically
    #[clap(short, long)]
    pub nonce: Option<FieldElement>,
}

#[allow(clippy::too_many_arguments)]
pub async fn deploy(
    class_hash: FieldElement,
    constructor_calldata: Vec<FieldElement>,
    salt: Option<FieldElement>,
    unique: bool,
    max_fee: Option<FieldElement>,
    account: &SingleOwnerAccount<&JsonRpcClient<HttpTransport>, LocalWallet>,
    nonce: Option<FieldElement>,
    wait_config: WaitForTx,
) -> Result<DeployResponse> {
    let salt = extract_or_generate_salt(salt);

    let factory = ContractFactory::new(class_hash, account);
    let deployment = factory.deploy(constructor_calldata.clone(), salt, unique);

    // TODO(#1396): use apply_optional here when `Deployment` in starknet-rs is public
    //  otherwise we cannot pass the necessary reference to a function
    let execution_with_fee = if let Some(max_fee) = max_fee {
        deployment.max_fee(max_fee)
    } else {
        deployment
    };

    let execution = if let Some(nonce) = nonce {
        execution_with_fee.nonce(nonce)
    } else {
        execution_with_fee
    };

    let result = execution.send().await;

    match result {
        Ok(result) => {
            handle_wait_for_tx(
                account.provider(),
                result.transaction_hash,
                DeployResponse {
                    contract_address: Hex(get_udc_deployed_address(
                        salt,
                        class_hash,
                        &udc_uniqueness(unique, account.address()),
                        &constructor_calldata,
                    )),
                    transaction_hash: Hex(result.transaction_hash),
                },
                wait_config,
            )
            .await
        }
        Err(Provider(error)) => handle_rpc_error(error),
        _ => Err(anyhow!("Unknown RPC error")),
    }
}
