use anyhow::{anyhow, Result};
use clap::Args;
use sncast::response::structs::{DeployResponse, Felt};
use starknet::accounts::AccountError::Provider;
use starknet::accounts::{Account, ConnectedAccount, SingleOwnerAccount};
use starknet::contract::ContractFactory;
use starknet::core::types::FieldElement;
use starknet::core::utils::get_udc_deployed_address;
use starknet::providers::jsonrpc::HttpTransport;
use starknet::providers::JsonRpcClient;
use starknet::signers::LocalWallet;

use sncast::response::errors::StarknetCommandError;
use sncast::{extract_or_generate_salt, udc_uniqueness};
use sncast::{handle_wait_for_tx, WaitForTx};

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
) -> Result<DeployResponse, StarknetCommandError> {
    let salt = extract_or_generate_salt(salt);
    let factory = ContractFactory::new(class_hash, account);
    let execution = factory.deploy_v1(constructor_calldata.clone(), salt, unique);

    // TODO(#1396): use apply_optional here when `Deployment` in starknet-rs is public
    //  otherwise we cannot pass the necessary reference to a function
    let execution = if let Some(max_fee) = max_fee {
        execution.max_fee(max_fee)
    } else {
        execution
    };

    let execution = if let Some(nonce) = nonce {
        execution.nonce(nonce)
    } else {
        execution
    };

    let result = execution.send().await;
    match result {
        Ok(result) => handle_wait_for_tx(
            account.provider(),
            result.transaction_hash,
            DeployResponse {
                contract_address: Felt(get_udc_deployed_address(
                    salt,
                    class_hash,
                    &udc_uniqueness(unique, account.address()),
                    &constructor_calldata,
                )),
                transaction_hash: Felt(result.transaction_hash),
            },
            wait_config,
        )
        .await
        .map_err(StarknetCommandError::from),
        Err(Provider(error)) => Err(StarknetCommandError::ProviderError(error.into())),
        _ => Err(anyhow!("Unknown RPC error").into()),
    }
}
