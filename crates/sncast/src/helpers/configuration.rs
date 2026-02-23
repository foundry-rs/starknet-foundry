use super::block_explorer;
use crate::helpers::config::get_global_config_path;
use crate::response::ui::UI;
use crate::{Network, PartialWaitParams, ValidatedWaitParams};
use anyhow::Result;
use camino::Utf8PathBuf;
use configuration::{Config, load_config, Override, merge_optional};
use serde::{Deserialize, Serialize};
use url::Url;
use crate::helpers::constants::DEFAULT_ACCOUNTS_FILE;

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
}

impl Override for NetworksConfig {
    fn override_with(&self, other: NetworksConfig) -> NetworksConfig {
        NetworksConfig {
            mainnet: other.mainnet.clone().or_else(|| self.mainnet.clone()),
            sepolia: other.sepolia.clone().or_else(|| self.sepolia.clone()),
            devnet: other.devnet.clone().or_else(|| self.devnet.clone()),
        }
    }
}

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct CastConfig {
    #[serde(default)]
    /// RPC url
    pub url: Option<Url>,

    #[serde(default)]
    pub network: Option<Network>,

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

// TODO(#4027)
impl CastConfig {
    pub fn validate(&self) -> anyhow::Result<()> {
        if self.block_explorer.unwrap_or_default() == block_explorer::Service::StarkScan {
            anyhow::bail!(
                "starkscan.co was terminated and `'StarkScan'` is no longer available. Please set `block-explorer` to `'Voyager'` or other explorer of your choice."
            )
        }

        match (&self.url, &self.network) {
            (Some(_), Some(_)) => {
                anyhow::bail!("Only one of `url` or `network` may be specified")
            }
            _ => Ok(()),
        }
    }
}

impl Default for CastConfig {
    fn default() -> Self {
        Self {
            url: None,
            network: None,
            account: String::default(),
            accounts_file: Utf8PathBuf::from(DEFAULT_ACCOUNTS_FILE),
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
        let config = serde_json::from_value::<Self>(config)?;
        config.validate()?;
        Ok(config)
    }
}

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Default)]
pub struct PartialCastConfig {
    pub url: Option<Url>,
    pub network: Option<Network>,
    pub account: Option<String>,
    #[serde(
        default,
        rename(serialize = "accounts-file", deserialize = "accounts-file")
    )]
    pub accounts_file: Option<Utf8PathBuf>,
    pub keystore: Option<Utf8PathBuf>,
    #[serde(
        default,
        rename(serialize = "wait-params", deserialize = "wait-params")
    )]
    pub wait_params: Option<PartialWaitParams>,
    #[serde(
        default,
        rename(serialize = "block-explorer", deserialize = "block-explorer")
    )]
    /// A block explorer service, used to display links to transaction details
    pub block_explorer: Option<block_explorer::Service>,

    #[serde(
        default,
        rename(serialize = "show-explorer-links", deserialize = "show-explorer-links")
    )]
    /// Print links pointing to pages with transaction details in the chosen block explorer
    pub show_explorer_links: Option<bool>,

    #[serde(default)]
    /// Configurable urls of predefined networks - mainnet, sepolia, and devnet are supported
    pub networks: Option<NetworksConfig>,
}

impl Config for PartialCastConfig {
    fn tool_name() -> &'static str {
        "sncast"
    }

    fn from_raw(config: serde_json::Value) -> Result<Self> {
        let config = serde_json::from_value::<Self>(config)?;
        // TODO: figure out whether that should be there at all
        config.validate()?;
        Ok(config)
    }
}

// TODO(#4027)
impl PartialCastConfig {
    pub fn validate(&self) -> anyhow::Result<()> {
        if self.block_explorer.unwrap_or_default() == block_explorer::Service::StarkScan {
            anyhow::bail!(
                "starkscan.co was terminated and `'StarkScan'` is no longer available. Please set `block-explorer` to `'Voyager'` or other explorer of your choice."
            )
        }

        match (&self.url, &self.network) {
            (Some(_), Some(_)) => {
                anyhow::bail!("Only one of `url` or `network` may be specified")
            }
            _ => Ok(()),
        }
    }
}

impl Override for PartialCastConfig {
    fn override_with(&self, other: PartialCastConfig) -> PartialCastConfig {
        PartialCastConfig {
            url: other.url.clone().or_else(|| self.url.clone()),
            network: other.network.or(self.network),
            account: other.account.clone().or_else(|| self.account.clone()),
            accounts_file: other
                .accounts_file
                .clone()
                .or_else(|| self.accounts_file.clone()),
            keystore: other.keystore.clone().or_else(|| self.keystore.clone()),
            wait_params: merge_optional(self.wait_params, other.wait_params),
            block_explorer: other.block_explorer.or(self.block_explorer),
            show_explorer_links: other.show_explorer_links.or(self.show_explorer_links),
            networks: merge_optional(self.networks.clone(), other.networks),
        }
    }
}

impl PartialCastConfig {
    pub fn local(opts: &CliConfigOpts) -> Result<Self> {
        let local_config = load_config::<PartialCastConfig>(None, opts.profile.as_deref())
            .map_err(|err| anyhow::anyhow!(format!("Failed to load config: {err}")))?;

        Ok(local_config)
    }

    pub fn global(opts: &CliConfigOpts, ui: &UI) -> Result<Self> {
        let global_config_path = get_global_config_path().unwrap_or_else(|err| {
            ui.print_error(&opts.command_name, format!("Error getting global config path: {err}"));
            Utf8PathBuf::new()
        });

        let global_config =
            load_config::<PartialCastConfig>(Some(&global_config_path.clone()), opts.profile.as_deref())
                .or_else(|_| load_config::<PartialCastConfig>(Some(&global_config_path), None))
                .map_err(|err| anyhow::anyhow!(format!("Failed to load config: {err}")))?;

        Ok(global_config)
    }
}

pub struct CliConfigOpts {
    pub command_name: String,
    pub profile: Option<String>,
}

impl From<PartialCastConfig> for CastConfig {
    fn from(partial: PartialCastConfig) -> Self {
        let default = CastConfig::default();
        let accounts_file = partial.accounts_file.unwrap_or(default.accounts_file);
        let accounts_file = Utf8PathBuf::from(shellexpand::tilde(&accounts_file).to_string());
        let networks = partial.networks.clone().map(|n| default.networks.override_with(n)).unwrap_or(default.networks);

        CastConfig {
            url: partial.url.or(default.url),
            network: partial.network.or(default.network),
            account: partial.account.unwrap_or(default.account),
            accounts_file,
            keystore: partial.keystore.or(default.keystore),
            wait_params: partial.wait_params.map(ValidatedWaitParams::from).unwrap_or(default.wait_params),
            block_explorer: partial.block_explorer.or(default.block_explorer),
            show_explorer_links: partial.show_explorer_links.unwrap_or(default.show_explorer_links),
            networks,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use url::Url;

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
        let global = NetworksConfig {
            mainnet: Some(Url::parse("https://global-mainnet.example.com").unwrap()),
            sepolia: Some(Url::parse("https://global-sepolia.example.com").unwrap()),
            devnet: None,
        };
        let local = NetworksConfig {
            mainnet: Some(Url::parse("https://local-mainnet.example.com").unwrap()),
            sepolia: None,
            devnet: None,
        };

        let overriden = global.override_with(local);

        // Local mainnet should override global
        assert_eq!(
            overriden.mainnet,
            Some(Url::parse("https://local-mainnet.example.com").unwrap())
        );
        // Global sepolia should remain
        assert_eq!(
            overriden.sepolia,
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
