use crate::consts::EXPECTED_RPC_VERSION;
use anyhow::{Context, Result};
use semver::{Version, VersionReq};
use starknet::providers::jsonrpc::HttpTransport;
use starknet::providers::{JsonRpcClient, Provider};
use std::str::FromStr;
use url::Url;

pub fn create_rpc_client(url: &str) -> Result<JsonRpcClient<HttpTransport>> {
    let parsed_url = Url::parse(url).with_context(|| format!("Failed to parse URL: {url}"))?;
    let client = JsonRpcClient::new(HttpTransport::new(parsed_url));
    Ok(client)
}

#[must_use]
pub fn is_expected_version(version: &Version) -> bool {
    // TODO(#3151): This is temporary adjustment as blast node returns 0.8.0-rc.3
    VersionReq::from_str(EXPECTED_RPC_VERSION)
        .expect("Failed to parse the expected RPC version")
        .matches(version)
        || VersionReq::from_str("0.8.0-rc.3")
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
