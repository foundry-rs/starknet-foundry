use anyhow::{anyhow, Result};
use clap::{Args, ValueEnum};
use scarb_api::StarknetContractArtifacts;
use sncast::helpers::error::token_not_supported_for_declaration;
use sncast::helpers::fee::{FeeArgs, FeeSettings, FeeToken, PayableTransaction};
use sncast::helpers::scarb_utils::CompiledContract;
use sncast::response::errors::StarknetCommandError;
use sncast::response::structs::DeclareResponse;
use sncast::response::structs::Felt;
use sncast::{apply_optional, handle_wait_for_tx, impl_payable_transaction, ErrorData, WaitForTx};
use starknet::accounts::AccountError;
use starknet::accounts::AccountError::Provider;
use starknet::accounts::{ConnectedAccount, DeclarationV2, DeclarationV3};
use starknet::core::types::DeclareTransactionResult;
use starknet::core::types::FieldElement;
use starknet::{
    accounts::{Account, SingleOwnerAccount},
    providers::jsonrpc::{HttpTransport, JsonRpcClient},
    signers::LocalWallet,
};
use std::collections::HashMap;
use std::sync::Arc;

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
    pub nonce: Option<FieldElement>,

    /// Specifies scarb package to be used
    #[clap(long)]
    pub package: Option<String>,

    /// Version of the declaration (can be inferred from fee token)
    #[clap(short, long)]
    pub version: Option<DeclareVersion>,
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

type SendDeclarationError<'client> = AccountError<<SingleOwnerAccount<&'client JsonRpcClient<HttpTransport>, LocalWallet> as starknet::accounts::Account>::SignError>;

async fn send_declaration<'client>(
    contract: CompiledContract,
    account: &'client SingleOwnerAccount<&'client JsonRpcClient<HttpTransport>, LocalWallet>,
    nonce: Option<FieldElement>,
    fee_settings: FeeSettings,
) -> Result<DeclareTransactionResult, SendDeclarationError<'client>> {
    let CompiledContract { class, hash } = contract;

    match fee_settings {
        FeeSettings::Eth { max_fee } => {
            let declaration = account.declare_v2(Arc::new(class), hash);

            let declaration = apply_optional(declaration, max_fee, DeclarationV2::max_fee);
            let declaration = apply_optional(declaration, nonce, DeclarationV2::nonce);

            declaration.send().await
        }

        FeeSettings::Strk {
            max_gas,
            max_gas_unit_price,
        } => {
            let declaration = account.declare_v3(Arc::new(class), hash);

            let declaration = apply_optional(declaration, max_gas, DeclarationV3::gas);
            let declaration =
                apply_optional(declaration, max_gas_unit_price, DeclarationV3::gas_price);
            let declaration = apply_optional(declaration, nonce, DeclarationV3::nonce);

            declaration.send().await
        }
    }
}

async fn handle_declaration<'client>(
    result: Result<DeclareTransactionResult, SendDeclarationError<'client>>,
    account: &'client SingleOwnerAccount<&'client JsonRpcClient<HttpTransport>, LocalWallet>,
    wait_config: WaitForTx,
) -> Result<DeclareResponse, StarknetCommandError> {
    match result {
        Ok(result) => {
            let wait = handle_wait_for_tx(
                account.provider(),
                result.transaction_hash,
                DeclareResponse {
                    class_hash: Felt(result.class_hash),
                    transaction_hash: Felt(result.transaction_hash),
                },
                wait_config,
            );

            wait.await.map_err(StarknetCommandError::from)
        }

        Err(Provider(error)) => Err(StarknetCommandError::ProviderError(error.into())),

        _ => Err(anyhow!("Unknown RPC error").into()),
    }
}

async fn get_fee_settings(
    declare: &Declare,
    account: &SingleOwnerAccount<&JsonRpcClient<HttpTransport>, LocalWallet>,
) -> Result<FeeSettings> {
    declare
        .fee_args
        .clone()
        .fee_token(declare.token_from_version())
        .try_into_fee_settings(account.provider(), account.block_id())
        .await
}

pub async fn declare_compiled(
    declare: Declare,
    account: &SingleOwnerAccount<&JsonRpcClient<HttpTransport>, LocalWallet>,
    contract: CompiledContract,
    wait_config: WaitForTx,
) -> Result<DeclareResponse, StarknetCommandError> {
    let fee_settings = get_fee_settings(&declare, account).await?;
    let declared = send_declaration(contract, account, declare.nonce, fee_settings).await;
    handle_declaration(declared, account, wait_config).await
}

pub async fn declare(
    declare: Declare,
    account: &SingleOwnerAccount<&JsonRpcClient<HttpTransport>, LocalWallet>,
    artifacts: &HashMap<String, StarknetContractArtifacts>,
    wait_config: WaitForTx,
) -> Result<DeclareResponse, StarknetCommandError> {
    let contract_artifacts =
        artifacts
            .get(&declare.contract)
            .ok_or(StarknetCommandError::ContractArtifactsNotFound(
                ErrorData::new(declare.contract.clone()),
            ))?;

    let contract = CompiledContract::from(contract_artifacts)?;

    declare_compiled(declare, account, contract, wait_config).await
}
