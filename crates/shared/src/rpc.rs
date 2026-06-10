use crate::consts::EXPECTED_RPC_VERSION;
use anyhow::{Context, Result};
use semver::{Version, VersionReq};
use starknet_rust::core::types::{BlockId, BlockTag};
use starknet_rust::providers::jsonrpc::HttpTransport;
use starknet_rust::providers::{JsonRpcClient, Provider};
use std::str::FromStr;
use url::Url;

pub fn create_rpc_client(url: &Url) -> Result<JsonRpcClient<HttpTransport>> {
    let client = JsonRpcClient::new(HttpTransport::new(url.clone()));
    Ok(client)
}

#[must_use]
pub fn is_expected_version(version: &Version) -> bool {
    let core_version = Version::new(version.major, version.minor, version.patch);
    VersionReq::from_str(EXPECTED_RPC_VERSION)
        .expect("Failed to parse the expected RPC version")
        .matches(&core_version)
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

#[cfg(test)]
mod tests {
    use super::is_expected_version;
    use semver::Version;

    #[test]
    fn matches_expected_release_version() {
        assert!(is_expected_version(&Version::parse("0.10.3").unwrap()));
    }

    #[test]
    fn matches_expected_prerelease_version() {
        assert!(is_expected_version(&Version::parse("0.10.3-rc.0").unwrap()));
    }

    #[test]
    fn matches_expected_build_metadata_version() {
        assert!(is_expected_version(
            &Version::parse("0.10.3+build.1").unwrap()
        ));
    }

    #[test]
    fn rejects_incompatible_prerelease_version() {
        assert!(!is_expected_version(
            &Version::parse("0.11.0-rc.0").unwrap()
        ));
    }
}
