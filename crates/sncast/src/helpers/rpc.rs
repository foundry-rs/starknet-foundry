use crate::helpers::configuration::CastConfig;
use crate::helpers::devnet::detection;
use crate::response::ui::UI;
use crate::{Network, get_provider};
use anyhow::{Result, bail};
use clap::Args;
use shared::consts::RPC_URL_VERSION;
use shared::verify_and_warn_if_incompatible_rpc_version;
use starknet_rust::providers::{JsonRpcClient, jsonrpc::HttpTransport};

#[derive(Args, Clone, Debug, Default)]
#[group(required = false, multiple = false)]
pub struct RpcArgs {
    /// RPC provider url address; overrides url from snfoundry.toml
    #[arg(short, long)]
    pub url: Option<String>,

    /// Use predefined network with a public provider. Note that this option may result in rate limits or other unexpected behavior.
    /// For devnet, attempts to auto-detect running starknet-devnet instance.
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

        let url = self.get_url(config).await?;

        assert!(!url.is_empty(), "url cannot be empty");
        let provider = get_provider(&url)?;

        // TODO(#3959) Remove `base_ui`
        verify_and_warn_if_incompatible_rpc_version(&provider, url, ui.base_ui()).await?;

        Ok(provider)
    }

    pub async fn get_url(&self, config: &CastConfig) -> Result<String> {
        match (&self.network, &self.url, &config.url) {
            (Some(network), None, _) => self.resolve_network_url(network, config).await,
            (None, Some(url), _) => Ok(url.clone()),
            (None, None, url) if !url.is_empty() => Ok(url.clone()),
            _ => bail!("Either `--network` or `--url` must be provided."),
        }
    }

    async fn resolve_network_url(&self, network: &Network, config: &CastConfig) -> Result<String> {
        if let Some(custom_url) = config.networks.get_url(*network) {
            Ok(custom_url.clone())
        } else {
            network.url(&FreeProvider::semi_random()).await
        }
    }
}

pub enum FreeProvider {
    Zan,
}

impl FreeProvider {
    #[must_use]
    pub fn semi_random() -> Self {
        Self::Zan
    }

    #[must_use]
    pub fn mainnet_rpc(&self) -> String {
        match self {
            FreeProvider::Zan => {
                format!("https://api.zan.top/public/starknet-mainnet/rpc/{RPC_URL_VERSION}")
            }
        }
    }

    #[must_use]
    pub fn sepolia_rpc(&self) -> String {
        match self {
            FreeProvider::Zan => {
                format!("https://api.zan.top/public/starknet-sepolia/rpc/{RPC_URL_VERSION}")
            }
        }
    }
}

impl Network {
    pub async fn url(self, provider: &FreeProvider) -> Result<String> {
        match self {
            Network::Mainnet => Ok(Self::free_mainnet_rpc(provider)),
            Network::Sepolia => Ok(Self::free_sepolia_rpc(provider)),
            Network::Devnet => Self::devnet_rpc(provider).await,
        }
    }

    fn free_mainnet_rpc(provider: &FreeProvider) -> String {
        provider.mainnet_rpc()
    }

    fn free_sepolia_rpc(provider: &FreeProvider) -> String {
        provider.sepolia_rpc()
    }

    async fn devnet_rpc(_provider: &FreeProvider) -> Result<String> {
        detection::detect_devnet_url()
            .await
            .map_err(|e| anyhow::anyhow!(e))
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
    use starknet_rust::providers::Provider;

    #[tokio::test]
    async fn test_mainnet_url_happy_case() {
        let provider = get_provider(&Network::free_mainnet_rpc(&FreeProvider::Zan)).unwrap();
        let spec_version = provider.spec_version().await.unwrap();
        assert!(is_expected_version(&Version::parse(&spec_version).unwrap()));
    }

    #[tokio::test]
    async fn test_sepolia_url_happy_case() {
        let provider = get_provider(&Network::free_sepolia_rpc(&FreeProvider::Zan)).unwrap();
        let spec_version = provider.spec_version().await.unwrap();
        assert!(is_expected_version(&Version::parse(&spec_version).unwrap()));
    }

    #[tokio::test]
    async fn test_custom_network_url_from_config() {
        let mut config = CastConfig::default();
        config.networks.mainnet =
            Some("https://starknet-mainnet.infura.io/v3/custom-api-key".to_string());
        config.networks.sepolia =
            Some("https://starknet-sepolia.g.alchemy.com/v2/custom-api-key".to_string());

        let rpc_args = RpcArgs {
            url: None,
            network: Some(Network::Mainnet),
        };

        let url = rpc_args.get_url(&config).await.unwrap();
        assert_eq!(
            url,
            "https://starknet-mainnet.infura.io/v3/custom-api-key".to_string()
        );
    }

    #[tokio::test]
    async fn test_fallback_to_default_network_url() {
        let config = CastConfig::default();

        let rpc_args = RpcArgs {
            url: None,
            network: Some(Network::Mainnet),
        };

        let url = rpc_args.get_url(&config).await.unwrap();
        assert_eq!(url, Network::free_mainnet_rpc(&FreeProvider::Zan));
    }
}
