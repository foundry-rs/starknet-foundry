use anyhow::{Context, Result, anyhow};
use clap::Args;
use conversions::IntoConv;
use conversions::byte_array::ByteArray;
use shared::rpc::get_starknet_version;
use sncast::helpers::artifacts::CastStarknetContractArtifacts;
use sncast::helpers::fee::{FeeArgs, FeeSettings};
use sncast::helpers::rpc::RpcArgs;
use sncast::response::declare::{
    AlreadyDeclaredResponse, DeclareResponse, DeclareTransactionResponse,
};
use sncast::response::errors::{SNCastProviderError, SNCastStarknetError, StarknetCommandError};
use sncast::response::ui::UI;
use sncast::{ErrorData, WaitForTx, apply_optional_fields, handle_wait_for_tx};
use starknet_rust::accounts::AccountError::Provider;
use starknet_rust::accounts::{ConnectedAccount, DeclarationV3};
use starknet_rust::core::types::{
    ContractExecutionError, DeclareTransactionResult, StarknetError, TransactionExecutionErrorData,
};
use starknet_rust::providers::ProviderError;
use starknet_rust::{
    accounts::{Account, SingleOwnerAccount},
    core::types::contract::{CompiledClass, SierraClass},
    providers::jsonrpc::{HttpTransport, JsonRpcClient},
    signers::LocalWallet,
};
use starknet_types_core::felt::Felt;
use std::collections::HashMap;
use std::sync::Arc;
use universal_sierra_compiler_api::compile_contract_sierra;

/// Common args shared by declare command variants.
#[derive(Args)]
pub struct DeclareCommonArgs {
    #[command(flatten)]
    pub fee_args: FeeArgs,

    /// Nonce of the transaction. If not provided, nonce will be set automatically
    #[arg(short, long)]
    pub nonce: Option<Felt>,

    #[command(flatten)]
    pub rpc: RpcArgs,
}

#[derive(Args)]
#[command(about = "Declare a contract to starknet", long_about = None)]
pub struct Declare {
    /// Contract name
    #[arg(short = 'c', long)]
    pub contract_name: String,

    /// Specifies scarb package to be used
    #[arg(long)]
    pub package: Option<String>,

    #[command(flatten)]
    pub common: DeclareCommonArgs,
}

// TODO(#3785)
#[expect(clippy::too_many_arguments)]
pub async fn declare(
    contract_name: String,
    fee_args: FeeArgs,
    nonce: Option<Felt>,
    account: &SingleOwnerAccount<&JsonRpcClient<HttpTransport>, LocalWallet>,
    artifacts: &HashMap<String, CastStarknetContractArtifacts>,
    wait_config: WaitForTx,
    skip_on_already_declared: bool,
    ui: &UI,
) -> Result<DeclareResponse, StarknetCommandError> {
    let contract_artifacts =
        artifacts
            .get(&contract_name)
            .ok_or(StarknetCommandError::ContractArtifactsNotFound(ErrorData {
                data: ByteArray::from(contract_name.as_str()),
            }))?;

    let contract_definition: SierraClass = serde_json::from_str(&contract_artifacts.sierra)
        .context("Failed to parse sierra artifact")?;
    let casm_contract_definition: CompiledClass =
        serde_json::from_str(&contract_artifacts.casm).context("Failed to parse casm artifact")?;

    declare_with_artifacts(
        contract_definition,
        casm_contract_definition,
        fee_args.clone(),
        nonce,
        account,
        wait_config,
        skip_on_already_declared,
        ui,
    )
    .await
}

#[allow(clippy::result_large_err)]
pub fn compile_sierra_to_casm(
    sierra_class: &SierraClass,
) -> Result<CompiledClass, StarknetCommandError> {
    let casm_json: String = serde_json::to_string(
        &compile_contract_sierra(
            &serde_json::to_value(sierra_class)
                .with_context(|| "Failed to convert sierra to JSON value".to_string())?,
        )
        .with_context(|| "Failed to compile sierra to casm".to_string())?,
    )
    .expect("serialization should succeed");

    let casm: CompiledClass = serde_json::from_str(&casm_json)
        .with_context(|| "Failed to deserialize casm JSON into CompiledClass".to_string())?;
    Ok(casm)
}

#[allow(clippy::too_many_arguments)]
pub async fn declare_with_artifacts(
    sierra_class: SierraClass,
    compiled_casm: CompiledClass,
    fee_args: FeeArgs,
    nonce: Option<Felt>,
    account: &SingleOwnerAccount<&JsonRpcClient<HttpTransport>, LocalWallet>,
    wait_config: WaitForTx,
    skip_on_already_declared: bool,
    ui: &UI,
) -> Result<DeclareResponse, StarknetCommandError> {
    let starknet_version = get_starknet_version(account.provider()).await?;
    let hash_function = CompiledClass::hash_function_from_starknet_version(&starknet_version)
        .ok_or(anyhow!("Unsupported Starknet version: {starknet_version}"))?;
    let casm_class_hash = compiled_casm
        .class_hash_with_hash_function(hash_function)
        .map_err(anyhow::Error::from)?;

    let class_hash = sierra_class.class_hash().map_err(anyhow::Error::from)?;

    let declaration = account.declare_v3(
        Arc::new(sierra_class.flatten().map_err(anyhow::Error::from)?),
        casm_class_hash,
    );

    let fee_settings = if fee_args.max_fee.is_some() {
        let fee_estimate = declaration
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

    let declaration = apply_optional_fields!(
        declaration,
        l1_gas => DeclarationV3::l1_gas,
        l1_gas_price => DeclarationV3::l1_gas_price,
        l2_gas => DeclarationV3::l2_gas,
        l2_gas_price => DeclarationV3::l2_gas_price,
        l1_data_gas => DeclarationV3::l1_data_gas,
        l1_data_gas_price => DeclarationV3::l1_data_gas_price,
        tip => DeclarationV3::tip,
        nonce => DeclarationV3::nonce
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
            ui,
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
        Err(Provider(ProviderError::StarknetError(StarknetError::ClassAlreadyDeclared))) => Err(
            StarknetCommandError::ProviderError(SNCastProviderError::StarknetError(
                SNCastStarknetError::ClassAlreadyDeclared(class_hash.into_()),
            )),
        ),
        Err(Provider(ProviderError::StarknetError(StarknetError::TransactionExecutionError(
            TransactionExecutionErrorData {
                execution_error: ContractExecutionError::Message(message),
                ..
            },
        )))) if message.contains("is already declared") => {
            if skip_on_already_declared {
                Ok(DeclareResponse::AlreadyDeclared(AlreadyDeclaredResponse {
                    class_hash: class_hash.into_(),
                }))
            } else {
                Err(StarknetCommandError::ProviderError(
                    SNCastProviderError::StarknetError(SNCastStarknetError::ClassAlreadyDeclared(
                        class_hash.into_(),
                    )),
                ))
            }
        }
        Err(Provider(error)) => Err(StarknetCommandError::ProviderError(error.into())),
        Err(error) => Err(anyhow!(format!("Unexpected error occurred: {error}")).into()),
    }
}
