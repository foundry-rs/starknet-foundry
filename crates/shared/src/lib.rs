use crate::consts::EXPECTED_RPC_VERSION;
use crate::print::print_as_warning;
use crate::rpc::{get_rpc_version, is_expected_version};
use anyhow::{anyhow, Result};
use starknet::providers::jsonrpc::HttpTransport;
use starknet::providers::JsonRpcClient;

pub mod command;
pub mod consts;
pub mod print;
pub mod rpc;
pub mod test_utils;
pub mod utils;

pub async fn verify_and_warn_if_incompatible_rpc_version(
    client: &JsonRpcClient<HttpTransport>,
    url: &str,
) -> Result<()> {
    let node_spec_version = get_rpc_version(client).await?;
    if !is_expected_version(&node_spec_version) {
        print_as_warning(&anyhow!(
            "RPC node with the url {url} uses incompatible version {node_spec_version}. Expected version: {EXPECTED_RPC_VERSION}"
        ));
    }

    Ok(())
}
