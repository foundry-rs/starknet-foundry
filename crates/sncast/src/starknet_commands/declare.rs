use anyhow::{anyhow, Context, Result};
use clap::Args;
use scarb_api::StarknetContractArtifacts;
use sncast::response::structs::DeclareResponse;
use sncast::response::structs::Felt;
use sncast::{apply_optional, handle_wait_for_tx, ErrorData, WaitForTx};
use starknet::accounts::AccountError::Provider;
use starknet::accounts::{ConnectedAccount, DeclarationV2};

use sncast::helpers::fee::FeeArgs;
use sncast::response::errors::StarknetCommandError;
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
    pub fee: FeeArgs,

    /// Nonce of the transaction. If not provided, nonce will be set automatically
    #[clap(short, long)]
    pub nonce: Option<FieldElement>,

    /// Specifies scarb package to be used
    #[clap(long)]
    pub package: Option<String>,
}

#[allow(clippy::too_many_lines)]
pub async fn declare(
    contract_name: &str,
    max_fee: Option<FieldElement>,
    account: &SingleOwnerAccount<&JsonRpcClient<HttpTransport>, LocalWallet>,
    nonce: Option<FieldElement>,
    artifacts: &HashMap<String, StarknetContractArtifacts>,
    wait_config: WaitForTx,
) -> Result<DeclareResponse, StarknetCommandError> {
    let contract_name: String = contract_name.to_string();
    let contract_artifacts =
        artifacts
            .get(&contract_name)
            .ok_or(StarknetCommandError::ContractArtifactsNotFound(
                ErrorData::new(contract_name),
            ))?;

    let contract_definition: SierraClass = serde_json::from_str(&contract_artifacts.sierra)
        .context("Failed to parse sierra artifact")?;
    let casm_contract_definition: CompiledClass =
        serde_json::from_str(&contract_artifacts.casm).context("Failed to parse casm artifact")?;

    let casm_class_hash = casm_contract_definition
        .class_hash()
        .map_err(anyhow::Error::from)?;

    let declaration = account.declare_v2(
        Arc::new(contract_definition.flatten().map_err(anyhow::Error::from)?),
        casm_class_hash,
    );

    let declaration = apply_optional(declaration, max_fee, DeclarationV2::max_fee);
    let declaration = apply_optional(declaration, nonce, DeclarationV2::nonce);
    let declared = declaration.send().await;
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
