use anyhow::{anyhow, Result};
use clap::{Args, ValueEnum};
use conversions::IntoConv;
use sncast::helpers::error::token_not_supported_for_deployment;
use sncast::helpers::fee::{FeeArgs, FeeSettings, FeeToken, PayableTransaction};
use sncast::helpers::rpc::RpcArgs;
use sncast::helpers::scarb_utils::{read_manifest_and_build_artifacts, CompiledContract};
use sncast::helpers::version::parse_version;
use sncast::response::errors::StarknetCommandError;
use sncast::response::structs::DeployResponse;
use sncast::{extract_or_generate_salt, impl_payable_transaction, udc_uniqueness};
use sncast::{handle_wait_for_tx, WaitForTx};
use starknet::accounts::AccountError::Provider;
use starknet::accounts::{Account, ConnectedAccount, SingleOwnerAccount};
use starknet::contract::ContractFactory;
use starknet::core::utils::get_udc_deployed_address;
use starknet::providers::jsonrpc::HttpTransport;
use starknet::providers::JsonRpcClient;
use starknet::signers::LocalWallet;
use starknet_types_core::felt::Felt;

use super::declare_deploy::DeclareDeploy;

#[derive(Args, Clone)]
#[command(about = "Deploy a contract on Starknet")]
pub struct Deploy {
    /// Class hash of contract to deploy
    #[clap(short = 'g', long, conflicts_with = "contract_name")]
    pub class_hash: Option<Felt>,

    // Name of the contract to deploy
    #[clap(long, conflicts_with = "class_hash")]
    pub contract_name: Option<String>,

    #[clap(flatten)]
    pub arguments: DeployArguments,

    #[clap(flatten)]
    pub fee_args: FeeArgs,

    #[clap(flatten)]
    pub rpc: RpcArgs,
}

#[derive(Debug, Clone, clap::Args)]
pub struct DeployArguments {
    // A package to deploy a contract from
    #[clap(long)]
    pub package: Option<String>,

    /// Arguments of the called function serialized as a series of felts
    #[clap(short, long, value_delimiter = ' ', num_args = 1..)]
    pub constructor_calldata: Option<Vec<String>>,

    /// Salt for the address
    #[clap(short, long)]
    pub salt: Option<Felt>,

    /// If true, salt will be modified with an account address
    #[clap(long)]
    pub unique: bool,

    /// Nonce of the transaction. If not provided, nonce will be set automatically
    #[clap(short, long)]
    pub nonce: Option<Felt>,

    /// Version of the deployment (can be inferred from fee token)
    #[clap(short, long, value_parser = parse_version::<DeployVersion>)]
    pub version: Option<DeployVersion>,

    // Arguments of the called function as a comma-separated string of Cairo expressions
    #[clap(long)]
    pub arguments: Option<String>,
}

#[derive(ValueEnum, Debug, Clone)]
pub enum DeployVersion {
    V1,
    V3,
}

impl From<DeclareDeploy> for Deploy {
    fn from(declare_deploy: DeclareDeploy) -> Self {
        let DeclareDeploy {
            contract_name,
            deploy_args,
            fee_token,
            rpc,
        } = declare_deploy;

        let fee_args = FeeArgs {
            fee_token: Some(fee_token),
            ..Default::default()
        };

        Deploy {
            class_hash: None,
            contract_name: Some(contract_name),
            arguments: deploy_args,
            fee_args,
            rpc,
        }
    }
}

impl Deploy {
    pub fn build_artifacts_and_get_compiled_contract(
        &self,
        json: bool,
        profile: Option<&String>,
    ) -> Result<CompiledContract> {
        let contract_name = self
            .contract_name
            .clone()
            .ok_or_else(|| anyhow!("Contract name and class hash unspecified"))?;

        let artifacts = read_manifest_and_build_artifacts(&self.arguments.package, json, profile)?;

        let contract_artifacts = artifacts
            .get(&contract_name)
            .ok_or_else(|| anyhow!("No artifacts found for contract: {}", contract_name))?;

        contract_artifacts.try_into()
    }

    pub fn resolved_with_class_hash(mut self, value: Felt) -> DeployResolved {
        self.class_hash = Some(value);
        self.try_into().unwrap()
    }
}

pub struct DeployResolved {
    pub class_hash: Felt,
    pub constructor_calldata: Vec<String>,
    pub salt: Option<Felt>,
    pub unique: bool,
    pub fee_args: FeeArgs,
    pub nonce: Option<Felt>,
    pub version: Option<DeployVersion>,
}

impl TryFrom<Deploy> for DeployResolved {
    type Error = anyhow::Error;

    fn try_from(deploy: Deploy) -> Result<Self, Self::Error> {
        let Deploy {
            class_hash,
            arguments:
                DeployArguments {
                    constructor_calldata,
                    salt,
                    unique,
                    nonce,
                    version,
                    ..
                },
            fee_args,
            ..
        } = deploy;

        let class_hash = class_hash.ok_or_else(|| anyhow!("Class hash unspecified"))?;

        Ok(DeployResolved {
            class_hash,
            constructor_calldata: constructor_calldata.unwrap_or_default(),
            salt,
            unique,
            fee_args,
            nonce,
            version,
        })
    }
}

impl_payable_transaction!(DeployResolved, token_not_supported_for_deployment,
    DeployVersion::V1 => FeeToken::Eth,
    DeployVersion::V3 => FeeToken::Strk
);

#[allow(clippy::ptr_arg, clippy::too_many_arguments)]
pub async fn deploy(
    class_hash: Felt,
    calldata: &Vec<Felt>,
    salt: Option<Felt>,
    unique: bool,
    fee_settings: FeeSettings,
    nonce: Option<Felt>,
    account: &SingleOwnerAccount<&JsonRpcClient<HttpTransport>, LocalWallet>,
    wait_config: WaitForTx,
) -> Result<DeployResponse, StarknetCommandError> {
    let salt = extract_or_generate_salt(salt);
    let factory = ContractFactory::new(class_hash, account);
    let result = match fee_settings {
        FeeSettings::Eth { max_fee } => {
            let execution = factory.deploy_v1(calldata.clone(), salt, unique);
            let execution = match max_fee {
                None => execution,
                Some(max_fee) => execution.max_fee(max_fee.into()),
            };
            let execution = match nonce {
                None => execution,
                Some(nonce) => execution.nonce(nonce),
            };
            execution.send().await
        }
        FeeSettings::Strk {
            max_gas,
            max_gas_unit_price,
        } => {
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
            execution.send().await
        }
    };

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
