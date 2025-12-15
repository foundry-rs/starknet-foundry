use super::block_explorer;
use crate::{Network, ValidatedWaitParams, helpers::config::RpcConfigWrapper};
use anyhow::Result;
use camino::Utf8PathBuf;
use configuration::Config;
use serde::{Deserialize, Serialize};
use url::Url;

#[must_use]
pub const fn show_explorer_links_default() -> bool {
    true
}

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Default)]
#[serde(deny_unknown_fields)]
pub struct NetworksConfig {
    pub mainnet: Option<Url>,
    pub sepolia: Option<Url>,
    pub devnet: Option<Url>,
}

impl NetworksConfig {
    #[must_use]
    pub fn get_url(&self, network: Network) -> Option<&Url> {
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
        Ok(serde_json::from_value::<Self>(config)?)
    }
}

#[cfg(test)]
mod tests {
    use url::Url;

    use super::*;

    #[test]
    fn test_networks_config_get() {
        let networks = NetworksConfig {
            mainnet: Some(Url::parse("https://mainnet.example.com").unwrap()),
            sepolia: Some(Url::parse("https://sepolia.example.com").unwrap()),
            devnet: Some(Url::parse("https://devnet.example.com").unwrap()),
        };

        assert_eq!(
            networks.get_url(Network::Mainnet),
            Some(&Url::parse("https://mainnet.example.com").unwrap())
        );
        assert_eq!(
            networks.get_url(Network::Sepolia),
            Some(&Url::parse("https://sepolia.example.com").unwrap())
        );
        assert_eq!(
            networks.get_url(Network::Devnet),
            Some(&Url::parse("https://devnet.example.com").unwrap())
        );
    }

    #[test]
    fn test_networks_config_override() {
        let mut global = NetworksConfig {
            mainnet: Some(Url::parse("https://global-mainnet.example.com").unwrap()),
            sepolia: Some(Url::parse("https://global-sepolia.example.com").unwrap()),
            devnet: None,
        };
        let local = NetworksConfig {
            mainnet: Some(Url::parse("https://local-mainnet.example.com").unwrap()),
            sepolia: None,
            devnet: None,
        };

        global.override_with(&local);

        // Local mainnet should override global
        assert_eq!(
            global.mainnet,
            Some(Url::parse("https://local-mainnet.example.com").unwrap())
        );
        // Global sepolia should remain
        assert_eq!(
            global.sepolia,
            Some(Url::parse("https://global-sepolia.example.com").unwrap())
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
