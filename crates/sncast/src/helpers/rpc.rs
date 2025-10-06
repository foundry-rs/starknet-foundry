use crate::helpers::configuration::CastConfig;
use crate::helpers::devnet_detection;
use crate::{Network, get_provider};
use anyhow::{Result, bail};
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

    /// Use predefined network with a public provider. Note that this option may result in rate limits or other unexpected behavior.
    /// For devnet, attempts to auto-detect running starknet-devnet instance. If auto-detection fails, use --url instead.
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

        let url = self.get_url(&config.url)?;

        assert!(!url.is_empty(), "url cannot be empty");
        let provider = get_provider(&url)?;

        verify_and_warn_if_incompatible_rpc_version(&provider, url, ui).await?;

        Ok(provider)
    }

    pub fn get_url(&self, config_url: &str) -> Result<String> {
        match (&self.network, &self.url, config_url.is_empty()) {
            (Some(network), None, _) => {
                let free_provider = FreeProvider::semi_random();
                network.url(&free_provider)
            }
            (None, Some(url), _) => Ok(url.clone()),
            (None, None, false) => Ok(config_url.to_string()),
            _ => bail!("Either `--network` or `--url` must be provided"),
        }
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
    pub fn url(self, provider: &FreeProvider) -> Result<String> {
        match self {
            Network::Mainnet => Ok(Self::free_mainnet_rpc(provider)),
            Network::Sepolia => Ok(Self::free_sepolia_rpc(provider)),
            Network::Devnet => Self::free_devnet_rpc(provider),
        }
    }

    fn free_mainnet_rpc(_provider: &FreeProvider) -> String {
        format!("https://starknet-mainnet.public.blastapi.io/rpc/{RPC_URL_VERSION}")
    }

    fn free_sepolia_rpc(_provider: &FreeProvider) -> String {
        format!("https://starknet-sepolia.public.blastapi.io/rpc/{RPC_URL_VERSION}")
    }

    fn free_devnet_rpc(_provider: &FreeProvider) -> Result<String> {
        devnet_detection::detect_devnet_url().map_err(|e| anyhow::anyhow!(e))
    }
}

#[must_use]
pub fn generate_network_flag(rpc_url: Option<&str>, network: Option<&Network>) -> String {
    if let Some(network) = network {
        format!("--network {network}")
    } else if let Some(rpc_url) = rpc_url {
        format!("--url {rpc_url}")
    } else {
        unreachable!("Either `--rpc_url` or `--network` must be provided.")
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
