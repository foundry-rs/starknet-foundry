use anyhow::{anyhow, Result};
use clap::{Args, ValueEnum};
use sncast::helpers::error::token_not_supported_for_deployment;
use sncast::helpers::fee::{FeeArgs, FeeSettings, FeeToken, PayableTransaction};
use sncast::response::errors::StarknetCommandError;
use sncast::response::structs::{DeployResponse, Felt};
use sncast::{extract_or_generate_salt, impl_payable_transaction, udc_uniqueness};
use sncast::{handle_wait_for_tx, WaitForTx};
use starknet::accounts::AccountError::Provider;
use starknet::accounts::{Account, ConnectedAccount, SingleOwnerAccount};
use starknet::contract::ContractFactory;
use starknet::core::types::FieldElement;
use starknet::core::utils::get_udc_deployed_address;
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
    #[clap(short, long, value_delimiter = ' ', num_args = 1..)]
    pub constructor_calldata: Vec<FieldElement>,

    /// Salt for the address
    #[clap(short, long)]
    pub salt: Option<FieldElement>,

    /// If true, salt will be modified with an account address
    #[clap(short, long)]
    pub unique: bool,

    #[clap(flatten)]
    pub fee_args: FeeArgs,

    /// Nonce of the transaction. If not provided, nonce will be set automatically
    #[clap(short, long)]
    pub nonce: Option<FieldElement>,

    /// Version of the deployment (can be inferred from fee token)
    #[clap(short, long)]
    pub version: Option<DeployVersion>,

    /// RPC provider url address; overrides url from snfoundry.toml
    #[clap(long = "url")]
    pub rpc_url: Option<String>,
}

#[derive(ValueEnum, Debug, Clone)]
pub enum DeployVersion {
    V1,
    V3,
}

impl_payable_transaction!(Deploy, token_not_supported_for_deployment,
    DeployVersion::V1 => FeeToken::Eth,
    DeployVersion::V3 => FeeToken::Strk
);

pub async fn deploy(
    deploy: Deploy,
    account: &SingleOwnerAccount<&JsonRpcClient<HttpTransport>, LocalWallet>,
    wait_config: WaitForTx,
) -> Result<DeployResponse, StarknetCommandError> {
    let fee_settings = deploy
        .fee_args
        .clone()
        .fee_token(deploy.token_from_version())
        .try_into_fee_settings(account.provider(), account.block_id())
        .await?;

    let salt = extract_or_generate_salt(deploy.salt);
    let factory = ContractFactory::new(deploy.class_hash, account);
    let result = match fee_settings {
        FeeSettings::Eth { max_fee } => {
            let execution =
                factory.deploy_v1(deploy.constructor_calldata.clone(), salt, deploy.unique);
            let execution = match max_fee {
                None => execution,
                Some(max_fee) => execution.max_fee(max_fee),
            };
            let execution = match deploy.nonce {
                None => execution,
                Some(nonce) => execution.nonce(nonce),
            };
            execution.send().await
        }
        FeeSettings::Strk {
            max_gas,
            max_gas_unit_price,
        } => {
            let execution =
                factory.deploy_v3(deploy.constructor_calldata.clone(), salt, deploy.unique);

            let execution = match max_gas {
                None => execution,
                Some(max_gas) => execution.gas(max_gas),
            };
            let execution = match max_gas_unit_price {
                None => execution,
                Some(max_gas_unit_price) => execution.gas_price(max_gas_unit_price),
            };
            let execution = match deploy.nonce {
                None => execution,
                Some(nonce) => execution.nonce(nonce),
            };
            execution.send().await
        }
    };

    match result {
        Ok(result) => handle_wait_for_tx(
            account.provider(),
            result.transaction_hash,
            DeployResponse {
                contract_address: Felt(get_udc_deployed_address(
                    salt,
                    deploy.class_hash,
                    &udc_uniqueness(deploy.unique, account.address()),
                    &deploy.constructor_calldata,
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
