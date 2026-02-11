use crate::starknet_commands::declare::{compile_casm_from_sierra, declare_with_artifacts};
use anyhow::Context;
use clap::Args;
use sncast::WaitForTx;
use sncast::helpers::fee::FeeArgs;
use sncast::helpers::rpc::RpcArgs;
use sncast::response::declare::DeclareResponse;
use sncast::response::errors::StarknetCommandError;
use sncast::response::ui::UI;
use starknet_rust::accounts::SingleOwnerAccount;
use starknet_rust::core::types::contract::SierraClass;
use starknet_rust::providers::jsonrpc::{HttpTransport, JsonRpcClient};
use starknet_rust::signers::LocalWallet;
use starknet_types_core::felt::Felt;
use std::path::PathBuf;

#[derive(Args)]
#[command(about = "Declare a contract to Starknet from a compiled Sierra file", long_about = None)]
pub struct DeclareFile {
    /// Path to the compiled Sierra contract class JSON file
    #[arg(long, short = 's')]
    pub sierra_file: PathBuf,

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
    let sierra_json = std::fs::read_to_string(&declare_file.sierra_file).with_context(|| {
        format!(
            "Failed to read Sierra file at {}",
            declare_file.sierra_file.display()
        )
    })?;

    let sierra: SierraClass = serde_json::from_str(&sierra_json)
        .with_context(|| "Failed to parse Sierra file as contract class".to_string())?;

    let casm = compile_casm_from_sierra(&sierra)?;

    declare_with_artifacts(
        sierra,
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
