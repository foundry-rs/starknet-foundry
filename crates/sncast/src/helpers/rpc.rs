use crate::helpers::configuration::CastConfig;
use crate::{Network, get_provider};
use anyhow::{Context, Result, bail};
use clap::Args;
use foundry_ui::UI;
use shared::consts::RPC_URL_VERSION;
use shared::verify_and_warn_if_incompatible_rpc_version;
use starknet::providers::{JsonRpcClient, jsonrpc::HttpTransport};
use url::Url;

#[derive(Args, Clone, Debug, Default)]
#[group(required = false, multiple = false)]
pub struct RpcArgs {
    /// RPC provider url address; overrides url from snfoundry.toml
    #[arg(short, long)]
    pub url: Option<String>,

    /// Use predefined network with a public provider. Note that this option may result in rate limits or other unexpected behavior
    #[arg(long)]
    pub network: Option<Network>,
}

impl RpcArgs {
    pub async fn get_provider(
        &self,
        config: &CastConfig,
        ui: &UI,
    ) -> Result<JsonRpcClient<HttpTransport>> {
        if self.network.is_some() && !config.url.is_empty() {
            bail!(
                "The argument '--network' cannot be used when `url` is defined in `snfoundry.toml` for the active profile"
            )
        }

        let url = self
            .get_url(&config.url)
            .context("Either `--network` or `--url` must be provided")?;

        assert!(!url.is_empty(), "url cannot be empty");
        let provider = get_provider(&url)?;

        verify_and_warn_if_incompatible_rpc_version(&provider, url, ui).await?;

        Ok(provider)
    }

    #[must_use]
    fn get_url(&self, config_url: &String) -> Option<String> {
        if let Some(network) = self.network {
            let free_provider = FreeProvider::semi_random();
            Some(network.url(&free_provider))
        } else {
            self.url.clone().or_else(|| {
                if config_url.is_empty() {
                    None
                } else {
                    Some(config_url.to_string())
                }
            })
        }
    }

    #[must_use]
    pub fn is_localhost(&self, config_url: &String) -> bool {
        self.get_url(config_url)
            .and_then(|url_str| Url::parse(&url_str).ok())
            .and_then(|url| url.host_str().map(str::to_string))
            .is_some_and(|host| host == "localhost" || host == "127.0.0.1" || host == "::1")
    }
}

pub enum FreeProvider {
    Blast,
}

impl FreeProvider {
    #[must_use]
    pub fn semi_random() -> Self {
        Self::Blast
    }
}

impl Network {
    #[must_use]
    pub fn url(self, provider: &FreeProvider) -> String {
        match self {
            Network::Mainnet => Self::free_mainnet_rpc(provider),
            Network::Sepolia => Self::free_sepolia_rpc(provider),
        }
    }

    fn free_mainnet_rpc(_provider: &FreeProvider) -> String {
        format!("https://starknet-mainnet.public.blastapi.io/rpc/{RPC_URL_VERSION}")
    }

    fn free_sepolia_rpc(_provider: &FreeProvider) -> String {
        format!("https://starknet-sepolia.public.blastapi.io/rpc/{RPC_URL_VERSION}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use semver::Version;
    use shared::rpc::is_expected_version;
    use starknet::providers::Provider;

    #[tokio::test]
    async fn test_mainnet_url_happy_case() {
        let provider = get_provider(&Network::free_mainnet_rpc(&FreeProvider::Blast)).unwrap();
        let spec_version = provider.spec_version().await.unwrap();
        assert!(is_expected_version(&Version::parse(&spec_version).unwrap()));
    }

    #[tokio::test]
    async fn test_sepolia_url_happy_case() {
        let provider = get_provider(&Network::free_sepolia_rpc(&FreeProvider::Blast)).unwrap();
        let spec_version = provider.spec_version().await.unwrap();
        assert!(is_expected_version(&Version::parse(&spec_version).unwrap()));
    }
}
