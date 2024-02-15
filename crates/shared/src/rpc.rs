use crate::consts::EXPECTED_RPC_VERSION;
use anyhow::Context;
use semver::Version;
use starknet::providers::jsonrpc::HttpTransport;
use starknet::providers::{JsonRpcClient, Provider};

#[must_use]
pub fn is_supported_version(version: &Version) -> bool {
    let expected_version =
        Version::parse(EXPECTED_RPC_VERSION).expect("Failed to parse the expected RPC version");
    *version == expected_version
}

pub async fn get_rpc_version(client: &JsonRpcClient<HttpTransport>) -> anyhow::Result<Version> {
    client
        .spec_version()
        .await
        .context("Error while calling RPC node")?
        .parse::<Version>()
        .context("Failed to parse RPC spec version")
}
