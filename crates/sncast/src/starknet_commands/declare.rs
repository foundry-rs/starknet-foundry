use anyhow::{anyhow, Context, Result};
use clap::{Args, ValueEnum};
use scarb_api::StarknetContractArtifacts;
use sncast::helpers::error::token_not_supported_for_declaration;
use sncast::helpers::fee::{FeeArgs, FeeSettings, FeeToken, PayableTransaction};
use sncast::helpers::rpc::RpcArgs;
use sncast::response::errors::StarknetCommandError;
use sncast::response::structs::DeclareResponse;
use sncast::response::structs::Felt;
use sncast::{apply_optional, handle_wait_for_tx, impl_payable_transaction, ErrorData, WaitForTx};
use starknet::accounts::AccountError::Provider;
use starknet::accounts::{ConnectedAccount, DeclarationV2, DeclarationV3};
use starknet::core::types::FieldElement;
use starknet::{
    accounts::{Account, SingleOwnerAccount},
    core::types::contract::{CompiledClass, SierraClass},
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

#[allow(clippy::too_many_lines)]
pub async fn declare(
    declare: Declare,
    account: &SingleOwnerAccount<&JsonRpcClient<HttpTransport>, LocalWallet>,
    artifacts: &HashMap<String, StarknetContractArtifacts>,
    wait_config: WaitForTx,
) -> Result<DeclareResponse, StarknetCommandError> {
    let fee_settings = declare
        .fee_args
        .clone()
        .fee_token(declare.token_from_version())
        .try_into_fee_settings(account.provider(), account.block_id())
        .await?;

    let contract_artifacts =
        artifacts
            .get(&declare.contract)
            .ok_or(StarknetCommandError::ContractArtifactsNotFound(
                ErrorData::new(declare.contract),
            ))?;

    let contract_definition: SierraClass = serde_json::from_str(&contract_artifacts.sierra)
        .context("Failed to parse sierra artifact")?;
    let casm_contract_definition: CompiledClass =
        serde_json::from_str(&contract_artifacts.casm).context("Failed to parse casm artifact")?;

    let casm_class_hash = casm_contract_definition
        .class_hash()
        .map_err(anyhow::Error::from)?;

    let declared = match fee_settings {
        FeeSettings::Eth { max_fee } => {
            let declaration = account.declare_v2(
                Arc::new(contract_definition.flatten().map_err(anyhow::Error::from)?),
                casm_class_hash,
            );

            let declaration = apply_optional(declaration, max_fee, DeclarationV2::max_fee);
            let declaration = apply_optional(declaration, declare.nonce, DeclarationV2::nonce);

            declaration.send().await
        }
        FeeSettings::Strk {
            max_gas,
            max_gas_unit_price,
        } => {
            let declaration = account.declare_v3(
                Arc::new(contract_definition.flatten().map_err(anyhow::Error::from)?),
                casm_class_hash,
            );

            let declaration = apply_optional(declaration, max_gas, DeclarationV3::gas);
            let declaration =
                apply_optional(declaration, max_gas_unit_price, DeclarationV3::gas_price);
            let declaration = apply_optional(declaration, declare.nonce, DeclarationV3::nonce);

            declaration.send().await
        }
    };

    match declared {
        Ok(result) => handle_wait_for_tx(
            account.provider(),
            result.transaction_hash,
            DeclareResponse {
                class_hash: Felt(result.class_hash),
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
