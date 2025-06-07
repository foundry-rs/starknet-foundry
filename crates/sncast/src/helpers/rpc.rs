use crate::helpers::configuration::CastConfig;
use crate::{Network, get_provider};
use anyhow::{Context, Result, bail};
use clap::Args;
use foundry_ui::UI;
use shared::consts::RPC_URL_VERSION;
use shared::verify_and_warn_if_incompatible_rpc_version;
use starknet::providers::{JsonRpcClient, jsonrpc::HttpTransport};

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

        let url = if let Some(network) = self.network {
            let free_provider = FreeProvider::semi_random();
            network.url(&free_provider)
        } else {
            let url = self.url.clone().or_else(|| {
                if config.url.is_empty() {
                    None
                } else {
                    Some(config.url.clone())
                }
            });

            url.context("Either `--network` or `--url` must be provided")?
        };

        assert!(!url.is_empty(), "url cannot be empty");
        let provider = get_provider(&url)?;

        verify_and_warn_if_incompatible_rpc_version(&provider, url, ui).await?;

        Ok(provider)
    }

    #[must_use]
    pub fn get_url(&self, config: &CastConfig) -> String {
        self.url.clone().unwrap_or_else(|| config.url.clone())
    }
}

enum FreeProvider {
    Blast,
}

impl FreeProvider {
    fn semi_random() -> Self {
        FreeProvider::Blast
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

    fn free_mainnet_rpc(provider: &FreeProvider) -> String {
        match provider {
            FreeProvider::Blast => {
                format!("https://starknet-mainnet.public.blastapi.io/rpc/{RPC_URL_VERSION}")
            }
        }
    }

    fn free_sepolia_rpc(provider: &FreeProvider) -> String {
        match provider {
            FreeProvider::Blast => {
                format!("https://starknet-sepolia.public.blastapi.io/rpc/{RPC_URL_VERSION}")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use semver::Version;
    use shared::rpc::is_expected_version;
    use starknet::providers::Provider;
    use test_case::test_case;

    #[test_case(FreeProvider::Blast)]
    #[tokio::test]
    async fn test_mainnet_url_happy_case(free_provider: FreeProvider) {
        let provider = get_provider(&Network::free_sepolia_rpc(&free_provider)).unwrap();
        let spec_version = provider.spec_version().await.unwrap();
        assert!(is_expected_version(&Version::parse(&spec_version).unwrap()));
    }

    #[test_case(FreeProvider::Blast)]
    #[tokio::test]
    async fn test_sepolia_url_happy_case(free_provider: FreeProvider) {
        let provider = get_provider(&Network::free_sepolia_rpc(&free_provider)).unwrap();
        let spec_version = provider.spec_version().await.unwrap();
        assert!(is_expected_version(&Version::parse(&spec_version).unwrap()));
    }
}
