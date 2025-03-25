use anyhow::{Result, anyhow};
use clap::Args;
use conversions::IntoConv;
use sncast::helpers::fee::{FeeArgs, FeeSettings};
use sncast::helpers::rpc::RpcArgs;
use sncast::response::errors::StarknetCommandError;
use sncast::response::structs::DeployResponse;
use sncast::{WaitForTx, handle_wait_for_tx};
use sncast::{extract_or_generate_salt, udc_uniqueness};
use starknet::accounts::AccountError::Provider;
use starknet::accounts::{Account, ConnectedAccount, SingleOwnerAccount};
use starknet::contract::ContractFactory;
use starknet::core::utils::get_udc_deployed_address;
use starknet::providers::JsonRpcClient;
use starknet::providers::jsonrpc::HttpTransport;
use starknet::signers::LocalWallet;
use starknet_types_core::felt::Felt;

#[derive(Args)]
#[command(about = "Deploy a contract on Starknet")]
pub struct Deploy {
    /// Class hash of contract to deploy
    #[arg(short = 'g', long)]
    pub class_hash: Felt,

    #[clap(flatten)]
    pub arguments: DeployArguments,

    /// Salt for the address
    #[arg(short, long)]
    pub salt: Option<Felt>,

    /// If true, salt will be modified with an account address
    #[arg(long)]
    pub unique: bool,

    #[clap(flatten)]
    pub fee_args: FeeArgs,

    /// Nonce of the transaction. If not provided, nonce will be set automatically
    #[arg(short, long)]
    pub nonce: Option<Felt>,

    #[clap(flatten)]
    pub rpc: RpcArgs,
}

#[derive(Debug, Clone, clap::Args)]
#[group(multiple = false)]
pub struct DeployArguments {
    /// Arguments of the called function serialized as a series of felts
    #[arg(short, long, value_delimiter = ' ', num_args = 1..)]
    pub constructor_calldata: Option<Vec<String>>,

    // Arguments of the called function as a comma-separated string of Cairo expressions
    #[arg(long)]
    pub arguments: Option<String>,
}

#[expect(clippy::ptr_arg, clippy::too_many_arguments)]
pub async fn deploy(
    class_hash: Felt,
    calldata: &Vec<Felt>,
    salt: Option<Felt>,
    unique: bool,
    fee_args: FeeArgs,
    nonce: Option<Felt>,
    account: &SingleOwnerAccount<&JsonRpcClient<HttpTransport>, LocalWallet>,
    wait_config: WaitForTx,
) -> Result<DeployResponse, StarknetCommandError> {
    let fee_settings = fee_args
        .try_into_fee_settings(account.provider(), account.block_id())
        .await?;

    let salt = extract_or_generate_salt(salt);
    let factory = ContractFactory::new(class_hash, account);

    let FeeSettings {
        max_gas,
        max_gas_unit_price,
    } = fee_settings;
    let execution = factory.deploy_v3(calldata.clone(), salt, unique);

    let execution = match max_gas {
        None => execution,
        Some(max_gas) => execution.gas(max_gas.into()),
    };
    let execution = match max_gas_unit_price {
        None => execution,
        Some(max_gas_unit_price) => execution.gas_price(max_gas_unit_price.into()),
    };
    let execution = match nonce {
        None => execution,
        Some(nonce) => execution.nonce(nonce),
    };
    let result = execution.send().await;

    match result {
        Ok(result) => handle_wait_for_tx(
            account.provider(),
            result.transaction_hash,
            DeployResponse {
                contract_address: get_udc_deployed_address(
                    salt,
                    class_hash,
                    &udc_uniqueness(unique, account.address()),
                    calldata,
                )
                .into_(),
                transaction_hash: result.transaction_hash.into_(),
            },
            wait_config,
        )
        .await
        .map_err(StarknetCommandError::from),
        Err(Provider(error)) => Err(StarknetCommandError::ProviderError(error.into())),
        Err(error) => Err(anyhow!(format!("Unexpected error occurred: {error}")).into()),
    }
}
