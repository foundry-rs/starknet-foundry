use crate::helpers::configuration::{CastConfig, NetworkParams};
use crate::helpers::devnet::detection;
use crate::{Network, get_provider};
use anyhow::{Result, bail};
use clap::Args;
use configuration::Override;
use foundry_ui::UI;
use semver::Version;
use shared::consts::{MAINNET_RPC_URL, SEPOLIA_RPC_URL};
use shared::verify_and_warn_if_incompatible_rpc_version;
use starknet_rust::providers::{JsonRpcClient, jsonrpc::HttpTransport};
use url::Url;

#[derive(Args, Clone, Debug, Default)]
#[group(required = false, multiple = false)]
pub struct RpcArgs {
    /// RPC provider url address; overrides url from snfoundry.toml
    #[arg(short, long)]
    pub url: Option<Url>,

    /// Use predefined network with a public provider. Note that this option may result in rate limits or other unexpected behavior.
    /// For devnet, attempts to auto-detect running starknet-devnet instance.
    #[arg(long)]
    pub network: Option<Network>,
}

impl RpcArgs {
    pub fn get_network_params(&self, config: &CastConfig) -> Result<NetworkParams> {
        let cli_params = NetworkParams::new(self.url.clone(), self.network)?;

        Ok(config.network_params.clone().override_with(cli_params))
    }

    pub async fn get_provider(
        &self,
        config: &CastConfig,
        ui: &UI,
    ) -> Result<JsonRpcClient<HttpTransport>> {
        let (provider, _) = self.get_provider_with_spec_version(config, ui).await?;
        Ok(provider)
    }

    pub async fn get_provider_with_spec_version(
        &self,
        config: &CastConfig,
        ui: &UI,
    ) -> Result<(JsonRpcClient<HttpTransport>, Version)> {
        let url = self.get_url(config).await?;
        let provider = get_provider(&url)?;

        let spec_version = verify_and_warn_if_incompatible_rpc_version(&provider, url, ui).await?;

        Ok((provider, spec_version))
    }

    pub async fn get_url(&self, config: &CastConfig) -> Result<Url> {
        let effective = self.get_network_params(config)?;

        match (effective.url(), effective.network()) {
            (Some(url), None) => Ok(url.clone()),
            (None, Some(network)) => self.resolve_network_url(&network, config).await,
            _ => bail!("Either `--network` or `--url` must be provided."),
        }
    }

    async fn resolve_network_url(&self, network: &Network, config: &CastConfig) -> Result<Url> {
        if let Some(custom_url) = config.networks.get_url(*network) {
            Ok(custom_url.clone())
        } else {
            network.url().await
        }
    }
}

impl Network {
    pub async fn url(self) -> Result<Url> {
        match self {
            Network::Mainnet => Ok(Url::parse(MAINNET_RPC_URL).expect("Failed to parse URL")),
            Network::Sepolia => Ok(Url::parse(SEPOLIA_RPC_URL).expect("Failed to parse URL")),
            Network::Devnet => detection::detect_devnet_url()
                .await
                .map_err(|e| anyhow::anyhow!(e)),
        }
    }
}

#[must_use]
pub fn generate_network_flag(rpc_args: &RpcArgs, config: &CastConfig) -> String {
    if let Some(network) = &rpc_args.network {
        format!("--network {network}")
    } else if let Some(rpc_url) = &rpc_args.url {
        format!("--url {rpc_url}")
    } else if config.network_params.url().is_some() || config.network_params.network().is_some() {
        // Since url or network is defined in config, no need to pass any flag
        String::new()
    } else {
        unreachable!(
            "`--url` or `--network` must be provided or one of url or network field must be set in snfoundry.toml"
        );
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
        let provider = get_provider(&Network::Mainnet.url().await.unwrap()).unwrap();
        let spec_version = provider.spec_version().await.unwrap();
        assert!(is_expected_version(&Version::parse(&spec_version).unwrap()));
    }

    #[tokio::test]
    async fn test_sepolia_url_happy_case() {
        let provider = get_provider(&Network::Sepolia.url().await.unwrap()).unwrap();
        let spec_version = provider.spec_version().await.unwrap();
        assert!(is_expected_version(&Version::parse(&spec_version).unwrap()));
    }

    #[tokio::test]
    async fn test_custom_network_url_from_config() {
        let mut config = CastConfig::default();
        config.networks.mainnet =
            Some(Url::parse("https://starknet-mainnet.infura.io/v3/custom-api-key").unwrap());
        config.networks.sepolia =
            Some(Url::parse("https://starknet-sepolia.g.alchemy.com/v2/custom-api-key").unwrap());

        let rpc_args = RpcArgs {
            url: None,
            network: Some(Network::Mainnet),
        };

        let url = rpc_args.get_url(&config).await.unwrap();
        assert_eq!(
            url,
            Url::parse("https://starknet-mainnet.infura.io/v3/custom-api-key").unwrap()
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
        assert_eq!(url.as_str(), MAINNET_RPC_URL);
    }
}
