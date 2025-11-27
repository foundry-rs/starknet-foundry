use crate::consts::EXPECTED_RPC_VERSION;
use anyhow::{Context, Result};
use semver::{Version, VersionReq};
use starknet_rust::core::types::{BlockId, BlockTag};
use starknet_rust::providers::jsonrpc::HttpTransport;
use starknet_rust::providers::{JsonRpcClient, Provider};
use std::str::FromStr;
use url::Url;

pub fn create_rpc_client(url: &str) -> Result<JsonRpcClient<HttpTransport>> {
    let parsed_url = Url::parse(url).with_context(|| format!("Failed to parse URL: {url}"))?;
    let client = JsonRpcClient::new(HttpTransport::new(parsed_url));
    Ok(client)
}

#[must_use]
pub fn is_expected_version(version: &Version) -> bool {
    VersionReq::from_str(EXPECTED_RPC_VERSION)
        .expect("Failed to parse the expected RPC version")
        .matches(version)
}

pub async fn get_rpc_version(client: &JsonRpcClient<HttpTransport>) -> Result<Version> {
    client
        .spec_version()
        .await
        .context("Error while calling RPC method spec_version")?
        .parse::<Version>()
        .context("Failed to parse RPC spec version")
}

pub async fn get_starknet_version(client: &JsonRpcClient<HttpTransport>) -> Result<String> {
    client
        .starknet_version(BlockId::Tag(BlockTag::PreConfirmed))
        .await
        .context("Error while getting Starknet version from the RPC provider")
}
