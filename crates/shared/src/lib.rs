use crate::consts::EXPECTED_RPC_VERSION;
use anyhow::{Context, Result};
use semver::Version;
use starknet::providers::{jsonrpc::HttpTransport, JsonRpcClient, Provider};

pub mod consts;
pub mod utils;

#[must_use]
pub fn is_supported_version(version: &Version) -> bool {
    let expected_version =
        Version::parse(EXPECTED_RPC_VERSION).expect("Failed to parse the expected RPC version");
    *version == expected_version
}

pub async fn get_and_parse_spec_version(client: &JsonRpcClient<HttpTransport>) -> Result<Version> {
    client
        .spec_version()
        .await
        .context("Error while calling RPC node")?
        .parse::<Version>()
        .context("Failed to parse RPC spec version")
}
