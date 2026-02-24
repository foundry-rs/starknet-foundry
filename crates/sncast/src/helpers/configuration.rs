use super::block_explorer;
use crate::helpers::config::get_global_config_path;
use crate::helpers::constants::DEFAULT_ACCOUNTS_FILE;
use crate::response::ui::UI;
use crate::{Network, PartialWaitParams, ValidatedWaitParams};
use anyhow::Result;
use camino::Utf8PathBuf;
use configuration::{Config, Override, load_config, override_optional};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use std::collections::BTreeMap;
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
}

impl Override for NetworksConfig {
    fn override_with(&self, other: NetworksConfig) -> NetworksConfig {
        NetworksConfig {
            mainnet: other.mainnet.or_else(|| self.mainnet.clone()),
            sepolia: other.sepolia.or_else(|| self.sepolia.clone()),
            devnet: other.devnet.or_else(|| self.devnet.clone()),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CastConfig {
    pub url: Option<Url>,
    pub network: Option<Network>,
    pub account: String,
    pub accounts_file: Utf8PathBuf,
    pub keystore: Option<Utf8PathBuf>,
    pub wait_params: ValidatedWaitParams,
    pub block_explorer: Option<block_explorer::Service>,
    pub show_explorer_links: bool,
    pub networks: NetworksConfig,
}

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

#[skip_serializing_none]
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
    pub networks: Option<NetworksConfig>,
}

#[derive(Serialize)]
pub struct SncastProfileAppend {
    pub sncast: BTreeMap<String, PartialCastConfig>,
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
            url: other.url.or_else(|| self.url.clone()),
            network: other.network.or(self.network),
            account: other.account.or_else(|| self.account.clone()),
            accounts_file: other.accounts_file.or_else(|| self.accounts_file.clone()),
            keystore: other.keystore.or_else(|| self.keystore.clone()),
            wait_params: override_optional(self.wait_params, other.wait_params),
            block_explorer: other.block_explorer.or(self.block_explorer),
            show_explorer_links: other.show_explorer_links.or(self.show_explorer_links),
            networks: override_optional(self.networks.clone(), other.networks),
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
            ui.print_error(
                &opts.command_name,
                format!("Error getting global config path: {err}"),
            );
            Utf8PathBuf::new()
        });

        let global_config = load_config::<PartialCastConfig>(
            Some(&global_config_path.clone()),
            opts.profile.as_deref(),
        )
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
    fn from(p: PartialCastConfig) -> Self {
        let d = CastConfig::default();

        let accounts_file = p.accounts_file.unwrap_or(d.accounts_file);
        let accounts_file = Utf8PathBuf::from(shellexpand::tilde(&accounts_file).to_string());

        let networks = p
            .networks
            .map(|n| d.networks.override_with(n))
            .unwrap_or(d.networks);

        CastConfig {
            url: p.url.or(d.url),
            network: p.network.or(d.network),
            account: p.account.unwrap_or(d.account),
            accounts_file,
            keystore: p.keystore.or(d.keystore),
            wait_params: p
                .wait_params
                .map_or(d.wait_params, ValidatedWaitParams::from),
            block_explorer: p.block_explorer.or(d.block_explorer),
            show_explorer_links: p.show_explorer_links.unwrap_or(d.show_explorer_links),
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

        let overridden = global.override_with(local);

        // Local mainnet should override global
        assert_eq!(
            overridden.mainnet,
            Some(Url::parse("https://local-mainnet.example.com").unwrap())
        );
        // Global sepolia should remain
        assert_eq!(
            overridden.sepolia,
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
