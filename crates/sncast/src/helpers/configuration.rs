use super::block_explorer;
use crate::ValidatedWaitParams;
use anyhow::Result;
use camino::Utf8PathBuf;
use configuration::Config;
use serde::{Deserialize, Serialize};

#[must_use]
pub const fn show_explorer_links_default() -> bool {
    true
}

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Default)]
#[serde(deny_unknown_fields)]
pub struct NetworksConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mainnet: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sepolia: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub devnet: Option<String>,
}

impl NetworksConfig {
    #[must_use]
    pub fn get(&self, network: &str) -> Option<&String> {
        match network {
            "mainnet" => self.mainnet.as_ref(),
            "sepolia" => self.sepolia.as_ref(),
            "devnet" => self.devnet.as_ref(),
            _ => None,
        }
    }

    pub fn merge(&mut self, other: &NetworksConfig) {
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
    #[serde(default)]
    /// RPC url
    pub url: String,

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
    /// Custom network URLs - mainnet, sepolia, and devnet are supported
    pub networks: NetworksConfig,
}

impl Default for CastConfig {
    fn default() -> Self {
        Self {
            url: String::default(),
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
            networks.get("mainnet"),
            Some(&"https://mainnet.example.com".to_string())
        );
        assert_eq!(
            networks.get("sepolia"),
            Some(&"https://sepolia.example.com".to_string())
        );
        assert_eq!(
            networks.get("devnet"),
            Some(&"https://devnet.example.com".to_string())
        );
        assert_eq!(networks.get("custom"), None);
    }

    #[test]
    fn test_networks_config_merge() {
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

        global.merge(&local);

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
            sepoila = "https://sepolia.example.com"
        "#;

        let result: Result<NetworksConfig, _> = toml::from_str(toml_str);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("unknown field"));
    }
}
