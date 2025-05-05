use anyhow::{Context, Result, anyhow};
use clap::Args;
use conversions::IntoConv;
use conversions::byte_array::ByteArray;
use scarb_api::StarknetContractArtifacts;
use sncast::helpers::fee::{FeeArgs, FeeSettings};
use sncast::helpers::rpc::RpcArgs;
use sncast::response::errors::StarknetCommandError;
use sncast::response::structs::{
    AlreadyDeclaredResponse, DeclareResponse, DeclareTransactionResponse,
};
use sncast::{ErrorData, WaitForTx, apply_optional_fields, handle_wait_for_tx};
use starknet::accounts::AccountError::Provider;
use starknet::accounts::{ConnectedAccount, DeclarationV3};
use starknet::core::types::{DeclareTransactionResult, StarknetError};
use starknet::providers::ProviderError;
use starknet::{
    accounts::{Account, SingleOwnerAccount},
    core::types::contract::{CompiledClass, SierraClass},
    providers::jsonrpc::{HttpTransport, JsonRpcClient},
    signers::LocalWallet,
};
use starknet_types_core::felt::Felt;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Args)]
#[command(about = "Declare a contract to starknet", long_about = None)]
pub struct Declare {
    /// Contract name
    #[arg(
        short = 'c',
        long = "contract-name",
        env = "SNCAST_DECLARE_CONTRACT_NAME"
    )]
    pub contract: String,

    #[command(flatten)]
    pub fee_args: FeeArgs,

    /// Nonce of the transaction. If not provided, nonce will be set automatically
    #[arg(short, long, env = "SNCAST_DECLARE_NONCE")]
    pub nonce: Option<Felt>,

    /// Specifies scarb package to be used
    #[arg(long, env = "SNCAST_DECLARE_PACKAGE")]
    pub package: Option<String>,

    #[command(flatten)]
    pub rpc: RpcArgs,
}

pub async fn declare(
    declare: Declare,
    account: &SingleOwnerAccount<&JsonRpcClient<HttpTransport>, LocalWallet>,
    artifacts: &HashMap<String, StarknetContractArtifacts>,
    wait_config: WaitForTx,
    skip_on_already_declared: bool,
) -> Result<DeclareResponse, StarknetCommandError> {
    let contract_artifacts =
        artifacts
            .get(&declare.contract)
            .ok_or(StarknetCommandError::ContractArtifactsNotFound(ErrorData {
                data: ByteArray::from(declare.contract.as_str()),
            }))?;

    let contract_definition: SierraClass = serde_json::from_str(&contract_artifacts.sierra)
        .context("Failed to parse sierra artifact")?;
    let casm_contract_definition: CompiledClass =
        serde_json::from_str(&contract_artifacts.casm).context("Failed to parse casm artifact")?;

    let casm_class_hash = casm_contract_definition
        .class_hash()
        .map_err(anyhow::Error::from)?;

    let class_hash = contract_definition
        .class_hash()
        .map_err(anyhow::Error::from)?;

    let declaration = account.declare_v3(
        Arc::new(contract_definition.flatten().map_err(anyhow::Error::from)?),
        casm_class_hash,
    );

    let fee_settings = if declare.fee_args.max_fee.is_some() {
        let fee_estimate = declaration
            .estimate_fee()
            .await
            .expect("Failed to estimate fee");
        declare.fee_args.try_into_fee_settings(Some(&fee_estimate))
    } else {
        declare.fee_args.try_into_fee_settings(None)
    };

    let FeeSettings {
        l1_gas,
        l1_gas_price,
        l2_gas,
        l2_gas_price,
        l1_data_gas,
        l1_data_gas_price,
    } = fee_settings.expect("Failed to convert to fee settings");

    let declaration = apply_optional_fields!(
        declaration,
        l1_gas => DeclarationV3::l1_gas,
        l1_gas_price => DeclarationV3::l1_gas_price,
        l2_gas => DeclarationV3::l2_gas,
        l2_gas_price => DeclarationV3::l2_gas_price,
        l1_data_gas => DeclarationV3::l1_data_gas,
        l1_data_gas_price => DeclarationV3::l1_data_gas_price,
        declare.nonce => DeclarationV3::nonce
    );

    let declared = declaration.send().await;

    match declared {
        Ok(DeclareTransactionResult {
            transaction_hash,
            class_hash,
        }) => handle_wait_for_tx(
            account.provider(),
            transaction_hash,
            DeclareResponse::Success(DeclareTransactionResponse {
                class_hash: class_hash.into_(),
                transaction_hash: transaction_hash.into_(),
            }),
            wait_config,
        )
        .await
        .map_err(StarknetCommandError::from),
        Err(Provider(ProviderError::StarknetError(StarknetError::ClassAlreadyDeclared)))
            if skip_on_already_declared =>
        {
            Ok(DeclareResponse::AlreadyDeclared(AlreadyDeclaredResponse {
                class_hash: class_hash.into_(),
            }))
        }
        Err(Provider(error)) => Err(StarknetCommandError::ProviderError(error.into())),
        Err(error) => Err(anyhow!(format!("Unexpected error occurred: {error}")).into()),
    }
}
