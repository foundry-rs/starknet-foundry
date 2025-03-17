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
    #[clap(short = 'g', long)]
    pub class_hash: Felt,

    #[clap(flatten)]
    pub arguments: DeployArguments,

    /// Salt for the address
    #[clap(short, long)]
    pub salt: Option<Felt>,

    /// If true, salt will be modified with an account address
    #[clap(long)]
    pub unique: bool,

    #[clap(flatten)]
    pub fee_args: FeeArgs,

    /// Nonce of the transaction. If not provided, nonce will be set automatically
    #[clap(short, long)]
    pub nonce: Option<Felt>,

    #[clap(flatten)]
    pub rpc: RpcArgs,
}

#[derive(Debug, Clone, clap::Args)]
#[group(multiple = false)]
pub struct DeployArguments {
    /// Arguments of the called function serialized as a series of felts
    #[clap(short, long, value_delimiter = ' ', num_args = 1..)]
    pub constructor_calldata: Option<Vec<String>>,

    // Arguments of the called function as a comma-separated string of Cairo expressions
    #[clap(long)]
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
    let salt = extract_or_generate_salt(salt);
    let factory = ContractFactory::new(class_hash, account);

    let execution = factory.deploy_v3(calldata.clone(), salt, unique);

    let fee_settings = if fee_args.max_fee.is_some() {
        let fee_estimate = execution
            .estimate_fee()
            .await
            .expect("Failed to estimate fee");
        fee_args.try_into_fee_settings(Some(&fee_estimate))
    } else {
        fee_args.try_into_fee_settings(None)
    };

    let FeeSettings {
        l1_gas,
        l1_gas_price,
        l2_gas,
        l2_gas_price,
        l1_data_gas,
        l1_data_gas_price,
    } = fee_settings.expect("Failed to convert to fee settings");

    let execution = match l1_gas {
        None => execution,
        Some(l1_gas) => execution.l1_gas(l1_gas),
    };
    let execution = match l1_gas_price {
        None => execution,
        Some(l1_gas_price) => execution.l1_gas_price(l1_gas_price),
    };
    let execution = match l2_gas {
        None => execution,
        Some(l2_gas) => execution.l2_gas(l2_gas),
    };
    let execution = match l2_gas_price {
        None => execution,
        Some(l2_gas_price) => execution.l2_gas_price(l2_gas_price),
    };
    let execution = match l1_data_gas {
        None => execution,
        Some(l1_data_gas) => execution.l1_data_gas(l1_data_gas),
    };
    let execution = match l1_data_gas_price {
        None => execution,
        Some(l1_data_gas_price) => execution.l1_data_gas_price(l1_data_gas_price),
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
