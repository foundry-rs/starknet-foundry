use anyhow::{Result, anyhow};
use clap::Args;
use conversions::IntoConv;
use sncast::helpers::fee::{FeeArgs, FeeSettings};
use sncast::helpers::rpc::RpcArgs;
use sncast::response::errors::StarknetCommandError;
use sncast::response::structs::DeployResponse;
use sncast::{WaitForTx, apply_optional_fields, handle_wait_for_tx};
use sncast::{extract_or_generate_salt, udc_uniqueness};
use starknet::accounts::AccountError::Provider;
use starknet::accounts::{Account, ConnectedAccount, SingleOwnerAccount};
use starknet::contract::{ContractFactory, DeploymentV3};
use starknet::core::utils::get_udc_deployed_address;
use starknet::providers::JsonRpcClient;
use starknet::providers::jsonrpc::HttpTransport;
use starknet::signers::LocalWallet;
use starknet_types_core::felt::Felt;

#[derive(Args)]
#[command(about = "Deploy a contract on Starknet")]
pub struct Deploy {
    /// Class hash of contract to deploy
    #[arg(short = 'g', long, env = "SNCAST_DEPLOY_CLASS_HASH")]
    pub class_hash: Felt,

    #[command(flatten)]
    pub arguments: DeployArguments,

    /// Salt for the address
    #[arg(short, long, env = "SNCAST_DEPLOY_SALT")]
    pub salt: Option<Felt>,

    /// If true, salt will be modified with an account address
    #[arg(long, env = "SNCAST_DEPLOY_UNIQUE")]
    pub unique: bool,

    #[command(flatten)]
    pub fee_args: FeeArgs,

    /// Nonce of the transaction. If not provided, nonce will be set automatically
    #[arg(short, long, env = "SNCAST_DEPLOY_NONCE")]
    pub nonce: Option<Felt>,

    #[command(flatten)]
    pub rpc: RpcArgs,
}

#[derive(Debug, Clone, clap::Args)]
#[group(multiple = false)]
pub struct DeployArguments {
    /// Arguments of the called function serialized as a series of felts
    #[arg(short, long, value_delimiter = ' ', num_args = 1.., env = "SNCAST_DEPLOY_CONSTRUCTOR_CALLDATA")]
    pub constructor_calldata: Option<Vec<String>>,

    // Arguments of the called function as a comma-separated string of Cairo expressions
    #[arg(long, env = "SNCAST_DEPLOY_ARGUMENTS")]
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

    let deployment = factory.deploy_v3(calldata.clone(), salt, unique);

    let fee_settings = if fee_args.max_fee.is_some() {
        let fee_estimate = deployment
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

    let deployment = apply_optional_fields!(
        deployment,
        l1_gas => DeploymentV3::l1_gas,
        l1_gas_price => DeploymentV3::l1_gas_price,
        l2_gas => DeploymentV3::l2_gas,
        l2_gas_price => DeploymentV3::l2_gas_price,
        l1_data_gas => DeploymentV3::l1_data_gas,
        l1_data_gas_price => DeploymentV3::l1_data_gas_price,
        nonce => DeploymentV3::nonce
    );
    let result = deployment.send().await;

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
