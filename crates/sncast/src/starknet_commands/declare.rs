use anyhow::{anyhow, Context, Result};
use clap::Args;
use scarb_api::StarknetContractArtifacts;
use sncast::response::structs::DeclareResponse;
use sncast::response::structs::Hex;
use sncast::{apply_optional, handle_rpc_error, handle_wait_for_tx, WaitForTx};
use starknet::accounts::AccountError::Provider;
use starknet::accounts::{ConnectedAccount, Declaration};

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

    /// Max fee for the transaction. If not provided, max fee will be automatically estimated
    #[clap(short, long)]
    pub max_fee: Option<FieldElement>,

    /// Nonce of the transaction. If not provided, nonce will be set automatically
    #[clap(short, long)]
    pub nonce: Option<FieldElement>,
}

#[allow(clippy::too_many_lines)]
pub async fn declare(
    contract_name: &str,
    max_fee: Option<FieldElement>,
    account: &SingleOwnerAccount<&JsonRpcClient<HttpTransport>, LocalWallet>,
    nonce: Option<FieldElement>,
    artifacts: &HashMap<String, StarknetContractArtifacts>,
    wait_config: WaitForTx,
) -> Result<DeclareResponse> {
    let contract_name: String = contract_name.to_string();
    let contract_artifacts = artifacts
        .get(&contract_name)
        .ok_or(anyhow!("Failed to find artifacts in starknet_artifacts.json file. Please ensure you have enabled sierra and casm code generation in Scarb.toml"))?;

    let contract_definition: SierraClass = serde_json::from_str(&contract_artifacts.sierra)
        .context("Failed to parse sierra artifact")?;
    let casm_contract_definition: CompiledClass =
        serde_json::from_str(&contract_artifacts.casm).context("Failed to parse casm artifact")?;

    let casm_class_hash = casm_contract_definition.class_hash()?;

    let declaration = account.declare(Arc::new(contract_definition.flatten()?), casm_class_hash);

    let declaration = apply_optional(declaration, max_fee, Declaration::max_fee);
    let declaration = apply_optional(declaration, nonce, Declaration::nonce);
    let declared = declaration.send().await;
    match declared {
        Ok(result) => {
            handle_wait_for_tx(
                account.provider(),
                result.transaction_hash,
                DeclareResponse {
                    class_hash: Hex(result.class_hash),
                    transaction_hash: Hex(result.transaction_hash),
                },
                wait_config,
            )
            .await
        }
        Err(Provider(error)) => handle_rpc_error(error),
        _ => Err(anyhow!("Unknown RPC error")),
    }
}
