use crate::consts::EXPECTED_RPC_VERSION;
use crate::rpc::{get_rpc_version, is_expected_version};
use anyhow::Result;
use foundry_ui::Ui;
use starknet::providers::JsonRpcClient;
use starknet::providers::jsonrpc::HttpTransport;
use std::fmt::Display;

pub mod auto_completions;
pub mod command;
pub mod consts;
pub mod rpc;
pub mod spinner;
pub mod test_utils;
pub mod utils;
pub mod vm;

pub async fn verify_and_warn_if_incompatible_rpc_version(
    client: &JsonRpcClient<HttpTransport>,
    url: impl Display,
    ui: &Ui,
) -> Result<()> {
    let node_spec_version = get_rpc_version(client).await?;
    if !is_expected_version(&node_spec_version) {
        ui.print_warning(&format!(
            "RPC node with the url {url} uses incompatible version {node_spec_version}. Expected version: {EXPECTED_RPC_VERSION}"
        ));
    }

    Ok(())
}
