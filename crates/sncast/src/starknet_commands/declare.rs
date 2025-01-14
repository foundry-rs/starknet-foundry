use anyhow::{anyhow, Result};
use clap::{Args, ValueEnum};
use conversions::byte_array::ByteArray;
use conversions::IntoConv;
use scarb_api::StarknetContractArtifacts;
use sncast::helpers::error::token_not_supported_for_declaration;
use sncast::helpers::fee::{FeeArgs, FeeSettings, FeeToken, PayableTransaction};
use sncast::helpers::rpc::RpcArgs;
use sncast::helpers::scarb_utils::CompiledContract;
use sncast::helpers::version::parse_version;
use sncast::response::errors::StarknetCommandError;
use sncast::response::structs::{
    AlreadyDeclaredResponse, DeclareResponse, DeclareTransactionResponse,
};
use sncast::{apply_optional, handle_wait_for_tx, impl_payable_transaction, ErrorData, WaitForTx};
use starknet::accounts::AccountError::Provider;
use starknet::accounts::{ConnectedAccount, DeclarationV2, DeclarationV3};
use starknet::core::types::{DeclareTransactionResult, StarknetError};
use starknet::providers::ProviderError;
use starknet::{
    accounts::{Account, SingleOwnerAccount},
    providers::jsonrpc::{HttpTransport, JsonRpcClient},
    signers::LocalWallet,
};
use starknet_types_core::felt::Felt;
use std::collections::HashMap;
use std::sync::Arc;

use super::declare_deploy::DeclareDeploy;
use super::deploy::DeployArguments;

#[derive(Args)]
#[command(about = "Declare a contract to starknet", long_about = None)]
pub struct Declare {
    /// Contract name
    #[clap(short = 'c', long = "contract-name")]
    pub contract: String,

    #[clap(flatten)]
    pub fee_args: FeeArgs,

    /// Nonce of the transaction. If not provided, nonce will be set automatically
    #[clap(short, long)]
    pub nonce: Option<Felt>,

    /// Specifies scarb package to be used
    #[clap(long)]
    pub package: Option<String>,

    /// Version of the declaration (can be inferred from fee token)
    #[clap(short, long, value_parser = parse_version::<DeclareVersion>)]
    pub version: Option<DeclareVersion>,

    #[clap(flatten)]
    pub rpc: RpcArgs,
}

#[derive(ValueEnum, Debug, Clone)]
pub enum DeclareVersion {
    V2,
    V3,
}

impl_payable_transaction!(Declare, token_not_supported_for_declaration,
    DeclareVersion::V2 => FeeToken::Eth,
    DeclareVersion::V3 => FeeToken::Strk
);

impl From<&DeclareDeploy> for Declare {
    fn from(declare_deploy: &DeclareDeploy) -> Self {
        let DeclareDeploy {
            contract_name,
            deploy_args: DeployArguments { package, .. },
            fee_token,
            rpc,
        } = &declare_deploy;

        let fee_args = FeeArgs {
            fee_token: Some(fee_token.to_owned()),
            ..Default::default()
        };

        Declare {
            contract: contract_name.to_owned(),
            fee_args,
            nonce: None,
            package: package.to_owned(),
            version: None,
            rpc: rpc.to_owned(),
        }
    }
}

#[allow(clippy::too_many_lines)]
pub async fn declare_compiled(
    declare: Declare,
    account: &SingleOwnerAccount<&JsonRpcClient<HttpTransport>, LocalWallet>,
    contract: CompiledContract,
    wait_config: WaitForTx,
    skip_on_already_declared: bool,
    fee_token: FeeToken,
) -> Result<DeclareResponse, StarknetCommandError> {
    let fee_settings = declare
        .fee_args
        .clone()
        .fee_token(fee_token)
        .try_into_fee_settings(account.provider(), account.block_id())
        .await?;

    let CompiledContract {
        class,
        casm_class_hash: class_hash,
        sierra_class_hash,
    } = contract;

    let result = match fee_settings {
        FeeSettings::Eth { max_fee } => {
            let declaration = account.declare_v2(Arc::new(class), class_hash);

            let declaration =
                apply_optional(declaration, max_fee.map(Felt::from), DeclarationV2::max_fee);
            let declaration = apply_optional(declaration, declare.nonce, DeclarationV2::nonce);

            declaration.send().await
        }

        FeeSettings::Strk {
            max_gas,
            max_gas_unit_price,
        } => {
            let declaration = account.declare_v3(Arc::new(class), class_hash);

            let declaration = apply_optional(
                declaration,
                max_gas.map(std::num::NonZero::get),
                DeclarationV3::gas,
            );
            let declaration = apply_optional(
                declaration,
                max_gas_unit_price.map(std::num::NonZero::get),
                DeclarationV3::gas_price,
            );
            let declaration = apply_optional(declaration, declare.nonce, DeclarationV3::nonce);

            declaration.send().await
        }
    };

    match result {
        Ok(DeclareTransactionResult {
            transaction_hash,
            class_hash,
        }) => {
            handle_wait_for_tx(
                account.provider(),
                transaction_hash,
                DeclareResponse::Success(DeclareTransactionResponse {
                    class_hash: class_hash.into_(),
                    transaction_hash: transaction_hash.into_(),
                }),
                wait_config,
            )
        }
        .await
        .map_err(StarknetCommandError::from),
        Err(Provider(ProviderError::StarknetError(StarknetError::ClassAlreadyDeclared)))
            if skip_on_already_declared =>
        {
            Ok(DeclareResponse::AlreadyDeclared(AlreadyDeclaredResponse {
                class_hash: sierra_class_hash.into_(),
            }))
        }
        Err(Provider(error)) => Err(StarknetCommandError::ProviderError(error.into())),
        Err(error) => Err(anyhow!(format!("Unexpected error occurred: {error}")).into()),
    }
}

pub async fn declare(
    declare: Declare,
    account: &SingleOwnerAccount<&JsonRpcClient<HttpTransport>, LocalWallet>,
    artifacts: &HashMap<String, StarknetContractArtifacts>,
    wait_config: WaitForTx,
) -> Result<DeclareResponse, StarknetCommandError> {
    let contract_artifacts = artifacts.get(&declare.contract).ok_or_else(|| {
        StarknetCommandError::ContractArtifactsNotFound(ErrorData {
            data: ByteArray::from(declare.contract.clone().as_str()),
        })
    })?;

    let contract = contract_artifacts.try_into()?;
    let fee_token = declare.fee_args.clone().fee_token.unwrap_or_default();

    declare_compiled(declare, account, contract, wait_config, true, fee_token).await
}
