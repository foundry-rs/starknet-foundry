use anyhow::{Result, anyhow};
use clap::Args;
use conversions::IntoConv;
use sncast::helpers::fee::{FeeArgs, FeeSettings};
use sncast::helpers::rpc::RpcArgs;
use sncast::response::deploy::StandardDeployResponse;
use sncast::response::errors::StarknetCommandError;
use sncast::response::ui::UI;
use sncast::{WaitForTx, apply_optional_fields, handle_wait_for_tx};
use sncast::{extract_or_generate_salt, udc_uniqueness};
use starknet_rust::accounts::AccountError::Provider;
use starknet_rust::accounts::{Account, ConnectedAccount, SingleOwnerAccount};
use starknet_rust::contract::{ContractFactory, DeploymentV3, UdcSelector};
use starknet_rust::core::utils::get_udc_deployed_address;
use starknet_rust::providers::JsonRpcClient;
use starknet_rust::providers::jsonrpc::HttpTransport;
use starknet_rust::signers::LocalWallet;
use starknet_types_core::felt::Felt;

#[derive(Args, Debug, Clone)]
#[group(required = true, multiple = false)]
pub struct ContractIdentifier {
    /// Class hash of contract to deploy
    #[arg(short = 'g', long)]
    pub class_hash: Option<Felt>,

    /// Contract name
    #[arg(long)]
    pub contract_name: Option<String>,
}

#[derive(Args)]
pub struct DeployCommonArgs {
    #[command(flatten)]
    pub contract_identifier: ContractIdentifier,

    #[command(flatten)]
    pub arguments: DeployArguments,

    /// Salt for the address
    #[arg(short, long)]
    pub salt: Option<Felt>,

    /// If true, salt will be modified with an account address
    #[arg(long)]
    pub unique: bool,
}

#[derive(Args)]
#[command(about = "Deploy a contract on Starknet")]
pub struct Deploy {
    #[command(flatten)]
    pub common: DeployCommonArgs,

    #[command(flatten)]
    pub fee_args: FeeArgs,

    /// Nonce of the transaction. If not provided, nonce will be set automatically
    #[arg(short, long)]
    pub nonce: Option<Felt>,

    #[command(flatten)]
    pub rpc: RpcArgs,

    /// Specifies scarb package to be used. Only possible to use with `--contract-name`.
    #[arg(long, conflicts_with_all = ["class_hash", "contract_name"])]
    pub package: Option<String>,
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
    ui: &UI,
) -> Result<StandardDeployResponse, StarknetCommandError> {
    let salt = extract_or_generate_salt(salt);

    // TODO(#3628): Use `ContractFactory::new` once new UDC address is the default one in starknet-rs
    let factory = ContractFactory::new_with_udc(class_hash, account, UdcSelector::New);

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
        tip,
    } = fee_settings.expect("Failed to convert to fee settings");

    let deployment = apply_optional_fields!(
        deployment,
        l1_gas => DeploymentV3::l1_gas,
        l1_gas_price => DeploymentV3::l1_gas_price,
        l2_gas => DeploymentV3::l2_gas,
        l2_gas_price => DeploymentV3::l2_gas_price,
        l1_data_gas => DeploymentV3::l1_data_gas,
        l1_data_gas_price => DeploymentV3::l1_data_gas_price,
        tip => DeploymentV3::tip,
        nonce => DeploymentV3::nonce
    );
    let result = deployment.send().await;

    match result {
        Ok(result) => handle_wait_for_tx(
            account.provider(),
            result.transaction_hash,
            StandardDeployResponse {
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
            ui,
        )
        .await
        .map_err(StarknetCommandError::from),
        Err(Provider(error)) => Err(StarknetCommandError::ProviderError(error.into())),
        Err(error) => Err(anyhow!(format!("Unexpected error occurred: {error}")).into()),
    }
}
