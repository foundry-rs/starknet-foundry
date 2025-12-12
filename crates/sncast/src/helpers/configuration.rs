use super::block_explorer;
use crate::{Network, ValidatedWaitParams, helpers::rpc::FreeProvider};
use anyhow::Result;
use camino::Utf8PathBuf;
use configuration::Config;
use serde::Deserializer;
use serde::de::Error;
use serde::{Deserialize, Serialize};
use url::Url;

#[must_use]
pub const fn show_explorer_links_default() -> bool {
    true
}

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Default)]
#[serde(deny_unknown_fields)]
pub struct NetworksConfig {
    pub mainnet: Option<String>,
    pub sepolia: Option<String>,
    pub devnet: Option<String>,
}

impl NetworksConfig {
    #[must_use]
    pub fn get_url(&self, network: Network) -> Option<&String> {
        match network {
            Network::Mainnet => self.mainnet.as_ref(),
            Network::Sepolia => self.sepolia.as_ref(),
            Network::Devnet => self.devnet.as_ref(),
        }
    }

    pub fn override_with(&mut self, other: &NetworksConfig) {
        if other.mainnet.is_some() {
            self.mainnet.clone_from(&other.mainnet);
        }
        if other.sepolia.is_some() {
            self.sepolia.clone_from(&other.sepolia);
        }
        if other.devnet.is_some() {
            self.devnet.clone_from(&other.devnet);
        }
    }
}

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
pub enum RpcConfig {
    Url(Url),
    Network(Network),
}

#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct RpcConfigWrapper {
    pub rpc_config: Option<RpcConfig>,
}

impl RpcConfig {
    pub async fn url(&self) -> Result<Url> {
        match self {
            RpcConfig::Url(url) => Ok(url.clone()),
            RpcConfig::Network(network) => {
                let url_str = network.url(&FreeProvider::semi_random()).await?;
                Ok(Url::parse(&url_str)?)
            }
        }
    }
}

impl<'de> Deserialize<'de> for RpcConfigWrapper {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Raw {
            url: Option<String>,
            network: Option<Network>,
        }

        let raw = Raw::deserialize(deserializer)?;

        match (raw.url, raw.network) {
            (Some(url), None) => {
                let parsed =
                    Url::parse(&url).map_err(|e| D::Error::custom(format!("Invalid URL: {e}")))?;
                Ok(Self {
                    rpc_config: Some(RpcConfig::Url(parsed)),
                })
            }
            (None, Some(net)) => Ok(Self {
                rpc_config: Some(RpcConfig::Network(net)),
            }),
            (None, None) => Ok(Self { rpc_config: None }),
            (Some(_), Some(_)) => Err(D::Error::custom(
                "Only one of `url` or `network` may be provided",
            )),
        }
    }
}

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct CastConfig {
    #[serde(flatten)]
    pub rpc_wrapper: RpcConfigWrapper,

    #[serde(default)]
    pub account: String,

    #[serde(
        default,
        rename(serialize = "accounts-file", deserialize = "accounts-file")
    )]
    pub accounts_file: Utf8PathBuf,

    pub keystore: Option<Utf8PathBuf>,

    #[serde(
        default,
        rename(serialize = "wait-params", deserialize = "wait-params")
    )]
    pub wait_params: ValidatedWaitParams,

    #[serde(
        default,
        rename(serialize = "block-explorer", deserialize = "block-explorer")
    )]
    /// A block explorer service, used to display links to transaction details
    pub block_explorer: Option<block_explorer::Service>,

    #[serde(
        default = "show_explorer_links_default",
        rename(serialize = "show-explorer-links", deserialize = "show-explorer-links")
    )]
    /// Print links pointing to pages with transaction details in the chosen block explorer
    pub show_explorer_links: bool,

    #[serde(default)]
    /// Configurable urls of predefined networks - mainnet, sepolia, and devnet are supported
    pub networks: NetworksConfig,
}

impl Default for CastConfig {
    fn default() -> Self {
        Self {
            rpc_wrapper: RpcConfigWrapper { rpc_config: None },
            account: String::default(),
            accounts_file: Utf8PathBuf::default(),
            keystore: None,
            wait_params: ValidatedWaitParams::default(),
            block_explorer: Some(block_explorer::Service::default()),
            show_explorer_links: show_explorer_links_default(),
            networks: NetworksConfig::default(),
        }
    }
}

impl Config for CastConfig {
    fn tool_name() -> &'static str {
        "sncast"
    }

    fn from_raw(config: serde_json::Value) -> Result<Self> {
        Ok(serde_json::from_value::<CastConfig>(config)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_networks_config_get() {
        let networks = NetworksConfig {
            mainnet: Some("https://mainnet.example.com".to_string()),
            sepolia: Some("https://sepolia.example.com".to_string()),
            devnet: Some("https://devnet.example.com".to_string()),
        };

        assert_eq!(
            networks.get_url(Network::Mainnet),
            Some(&"https://mainnet.example.com".to_string())
        );
        assert_eq!(
            networks.get_url(Network::Sepolia),
            Some(&"https://sepolia.example.com".to_string())
        );
        assert_eq!(
            networks.get_url(Network::Devnet),
            Some(&"https://devnet.example.com".to_string())
        );
    }

    #[test]
    fn test_networks_config_override() {
        let mut global = NetworksConfig {
            mainnet: Some("https://global-mainnet.example.com".to_string()),
            sepolia: Some("https://global-sepolia.example.com".to_string()),
            devnet: None,
        };
        let local = NetworksConfig {
            mainnet: Some("https://local-mainnet.example.com".to_string()),
            sepolia: None,
            devnet: None,
        };

        global.override_with(&local);

        // Local mainnet should override global
        assert_eq!(
            global.mainnet,
            Some("https://local-mainnet.example.com".to_string())
        );
        // Global sepolia should remain
        assert_eq!(
            global.sepolia,
            Some("https://global-sepolia.example.com".to_string())
        );
    }

    #[test]
    fn test_networks_config_rejects_unknown_fields_and_typos() {
        // Unknown fields should cause an error
        let toml_str = r#"
            mainnet = "https://mainnet.example.com"
            custom = "https://custom.example.com"
            wrong_key = "https://sepolia.example.com"
        "#;

        let result: Result<NetworksConfig, _> = toml::from_str(toml_str);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("unknown field"));
    }
}
