use crate::consts::EXPECTED_RPC_VERSION;
use crate::rpc::{get_rpc_version, is_expected_version};
use anyhow::Result;
use foundry_ui::UI;
use foundry_ui::components::warning::WarningMessage;
use semver::VersionReq;
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
    ui: &UI,
) -> Result<()> {
    let node_spec_version = get_rpc_version(client).await?;

    // TODO: (#3937) New RPC URL is not available yet
    if std::env::var("SNCAST_IGNORE_RPC_0_9_CHECK").is_ok_and(|v| v == "1")
        && VersionReq::parse("0.9.0")
            .expect("Failed to parse the expected RPC version")
            .matches(&node_spec_version)
    {
        return Ok(());
    }

    if !is_expected_version(&node_spec_version) {
        ui.println(&WarningMessage::new(&format!(
            "RPC node with the url {url} uses incompatible version {node_spec_version}. Expected version: {EXPECTED_RPC_VERSION}")));
    }

    Ok(())
}
