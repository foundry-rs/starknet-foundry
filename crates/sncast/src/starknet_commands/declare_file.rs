use crate::starknet_commands::declare::declare_with_artifacts;
use anyhow::Context;
use clap::Args;
use sncast::WaitForTx;
use sncast::helpers::fee::FeeArgs;
use sncast::helpers::rpc::RpcArgs;
use sncast::response::declare::DeclareResponse;
use sncast::response::errors::StarknetCommandError;
use sncast::response::ui::UI;
use starknet_rust::accounts::SingleOwnerAccount;
use starknet_rust::core::types::contract::{CompiledClass, SierraClass};
use starknet_rust::providers::jsonrpc::{HttpTransport, JsonRpcClient};
use starknet_rust::signers::LocalWallet;
use starknet_types_core::felt::Felt;
use std::path::PathBuf;
use universal_sierra_compiler_api::compile_contract_sierra;

#[derive(Args)]
#[command(about = "Declare a contract to Starknet from a compiled Sierra file", long_about = None)]
pub struct DeclareFile {
    /// Path to the compiled Sierra contract class JSON file
    #[arg(long)]
    pub sierra_path: PathBuf,

    #[command(flatten)]
    pub fee_args: FeeArgs,

    /// Nonce of the transaction. If not provided, nonce will be set automatically
    #[arg(short, long)]
    pub nonce: Option<Felt>,

    #[command(flatten)]
    pub rpc: RpcArgs,
}

pub async fn declare_file(
    declare_file: DeclareFile,
    account: &SingleOwnerAccount<&JsonRpcClient<HttpTransport>, LocalWallet>,
    wait_config: WaitForTx,
    skip_on_already_declared: bool,
    ui: &UI,
) -> Result<DeclareResponse, StarknetCommandError> {
    let sierra_json = std::fs::read_to_string(&declare_file.sierra_path).with_context(|| {
        format!(
            "Failed to read Sierra file at {}",
            declare_file.sierra_path.display()
        )
    })?;

    let contract_definition: SierraClass = serde_json::from_str(&sierra_json)
        .with_context(|| "Failed to parse Sierra file as contract class".to_string())?;

    let casm_json: String = serde_json::to_string(
        &compile_contract_sierra(
            &serde_json::to_value(&contract_definition)
                .with_context(|| "Failed to convert sierra to json value".to_string())?,
        )
        .with_context(|| "Failed to compile sierra to casm".to_string())?,
    )
    .expect("serialization should succeed");
    let casm: CompiledClass = serde_json::from_str(&casm_json)
        .with_context(|| "Failed to deserialize compiled CASM".to_string())?;

    declare_with_artifacts(
        contract_definition,
        casm,
        declare_file.fee_args,
        declare_file.nonce,
        account,
        wait_config,
        skip_on_already_declared,
        ui,
    )
    .await
}
