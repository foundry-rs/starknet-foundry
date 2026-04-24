use super::block_explorer;
use crate::helpers::constants::DEFAULT_ACCOUNTS_FILE;
use crate::{Network, PartialWaitParams, ValidatedWaitParams};
use anyhow::{Context, Result};
use camino::Utf8PathBuf;
use configuration::{Config, Override, load_config, override_optional};
use serde::de::IntoDeserializer;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use std::collections::{BTreeMap, HashMap};
use std::fmt::{Display, Formatter};
use url::Url;

#[must_use]
pub const fn show_explorer_links_default() -> bool {
    true
}

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Default)]
pub struct NetworkParams {
    url: Option<Url>,
    network: Option<Network>,
}

impl NetworkParams {
    pub fn new(url: Option<Url>, network: Option<Network>) -> Result<Self> {
        let res = Self { url, network };
        res.validate()?;
        Ok(res)
    }

    #[must_use]
    pub fn url(&self) -> Option<&Url> {
        self.url.as_ref()
    }

    #[must_use]
    pub fn network(&self) -> Option<Network> {
        self.network
    }

    pub fn validate(&self) -> Result<()> {
        match (&self.url, &self.network) {
            (Some(_), Some(_)) => anyhow::bail!("Only one of `url` or `network` may be specified"),
            _ => Ok(()),
        }
    }
}

impl Override for NetworkParams {
    fn override_with(&self, other: NetworkParams) -> NetworkParams {
        if other == NetworkParams::default() {
            self.clone()
        } else {
            other
        }
    }
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

/// Effective config used at runtime.
/// Note: Built from [`PartialCastConfig`], not (de)serialized.
#[derive(Clone, Debug, PartialEq)]
pub struct CastConfig {
    pub network_params: NetworkParams,
    pub account: String,
    pub accounts_file: Utf8PathBuf,
    pub keystore: Option<Utf8PathBuf>,
    pub wait_params: ValidatedWaitParams,
    pub block_explorer: Option<block_explorer::Service>,
    pub show_explorer_links: bool,
    pub networks: NetworksConfig,
    pub scarb_profile: String,
}

impl CastConfig {
    pub fn validate(&self) -> anyhow::Result<()> {
        block_explorer::Service::validate_for_config(self.block_explorer)?;
        self.wait_params.validate()?;
        self.network_params.validate()?;
        Ok(())
    }
}

impl Default for CastConfig {
    fn default() -> Self {
        Self {
            network_params: NetworkParams::default(),
            account: String::default(),
            accounts_file: Utf8PathBuf::from(DEFAULT_ACCOUNTS_FILE),
            keystore: None,
            wait_params: ValidatedWaitParams::default(),
            block_explorer: Some(block_explorer::Service::default()),
            show_explorer_links: show_explorer_links_default(),
            networks: NetworksConfig::default(),
            scarb_profile: "release".to_string(),
        }
    }
}

#[skip_serializing_none]
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Default)]
pub struct PartialCastConfig {
    #[serde(flatten)]
    pub network_params: NetworkParams,

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

    #[serde(
        default,
        rename(serialize = "scarb-profile", deserialize = "scarb-profile")
    )]
    pub scarb_profile: Option<String>,

    /// Additional data not captured by deserializer.
    #[doc(hidden)]
    #[serde(flatten, default, skip_serializing)]
    pub unknown_fields: HashMap<String, serde_json::Value>,
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
        let deserializer = config.into_deserializer();
        let config: Self = serde_path_to_error::deserialize(deserializer).map_err(|err| {
            let path_to_field = err.path().to_string();
            anyhow::anyhow!(
                "Failed to parse field `{path_to_field}`: {}",
                err.into_inner()
            )
        })?;
        config.validate()?;
        Ok(config)
    }
}

impl PartialCastConfig {
    pub fn validate(&self) -> anyhow::Result<()> {
        if !self.unknown_fields.is_empty() {
            let mut keys: Vec<&String> = self.unknown_fields.keys().collect();
            keys.sort();
            anyhow::bail!("unknown field(s) {keys:?}");
        }
        block_explorer::Service::validate_for_config(self.block_explorer)?;
        if let Some(ref wp) = self.wait_params {
            wp.validate()?;
        }
        self.network_params.validate()?;
        Ok(())
    }
}

impl Override for PartialCastConfig {
    fn override_with(&self, other: PartialCastConfig) -> PartialCastConfig {
        PartialCastConfig {
            network_params: self.network_params.override_with(other.network_params),
            account: other.account.or_else(|| self.account.clone()),
            accounts_file: other.accounts_file.or_else(|| self.accounts_file.clone()),
            keystore: other.keystore.or_else(|| self.keystore.clone()),
            wait_params: override_optional(self.wait_params, other.wait_params),
            block_explorer: other.block_explorer.or(self.block_explorer),
            show_explorer_links: other.show_explorer_links.or(self.show_explorer_links),
            networks: override_optional(self.networks.clone(), other.networks),
            scarb_profile: other.scarb_profile.or_else(|| self.scarb_profile.clone()),
            unknown_fields: HashMap::default(),
        }
    }
}

#[derive(Debug)]
pub enum MaybeConfig {
    NoFile,
    NoProfile,
    Loaded(Box<PartialCastConfig>),
}

impl MaybeConfig {
    #[must_use]
    pub fn unwrap_or_default(self) -> PartialCastConfig {
        match self {
            Self::NoFile | Self::NoProfile => PartialCastConfig::default(),
            Self::Loaded(config) => *config,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum ConfigScope {
    Local,
    Global,
}

impl Display for ConfigScope {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Local => write!(f, "local"),
            Self::Global => write!(f, "global"),
        }
    }
}

impl PartialCastConfig {
    fn load(path: &Utf8PathBuf, profile: Option<&str>, scope: ConfigScope) -> Result<Option<Self>> {
        load_config::<Self>(path, profile)
            .with_context(|| format!("Failed to load {scope} config at {path}"))
    }

    pub fn load_maybe(
        path: Option<&Utf8PathBuf>,
        profile: Option<&str>,
        scope: ConfigScope,
    ) -> Result<MaybeConfig> {
        match path {
            None => Ok(MaybeConfig::NoFile),
            Some(p) => match Self::load(p, profile, scope)? {
                None => Ok(MaybeConfig::NoProfile),
                Some(config) => Ok(MaybeConfig::Loaded(Box::from(config))),
            },
        }
    }
}

pub struct CliConfigOpts {
    pub command_name: String,
    pub profile: Option<String>,
}

impl TryFrom<PartialCastConfig> for CastConfig {
    type Error = anyhow::Error;

    /// Validates the config and returns a [`CastConfig`].
    fn try_from(p: PartialCastConfig) -> Result<Self> {
        let d = CastConfig::default();

        let accounts_file = p.accounts_file.unwrap_or(d.accounts_file);
        let accounts_file = Utf8PathBuf::from(shellexpand::tilde(&accounts_file).to_string());

        let networks = p
            .networks
            .map(|n| d.networks.override_with(n))
            .unwrap_or(d.networks);

        let config = CastConfig {
            network_params: d.network_params.override_with(p.network_params),
            account: p.account.unwrap_or(d.account),
            accounts_file,
            keystore: p.keystore.or(d.keystore),
            wait_params: p
                .wait_params
                .map_or_else(|| Ok(d.wait_params), ValidatedWaitParams::try_from)?,
            block_explorer: p.block_explorer.or(d.block_explorer),
            show_explorer_links: p.show_explorer_links.unwrap_or(d.show_explorer_links),
            networks,
            scarb_profile: p.scarb_profile.unwrap_or(d.scarb_profile),
        };
        config.validate()?;
        Ok(config)
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
    fn test_network_params_validation() {
        let url = Some(Url::parse("https://example.com").unwrap());
        let network = Some(Network::Sepolia);

        assert!(NetworkParams::new(url.clone(), network).is_err());
        assert!(NetworkParams::new(None, None).is_ok());
        assert!(NetworkParams::new(url, None).is_ok());
        assert!(NetworkParams::new(None, Some(Network::Mainnet)).is_ok());
    }

    #[test]
    fn test_network_params_override() {
        let global = NetworkParams::new(
            Some(Url::parse("https://global-sepolia.example.com").unwrap()),
            None,
        )
        .unwrap();
        let local = NetworkParams::new(None, Some(Network::Sepolia)).unwrap();
        let overridden = global.override_with(local.clone());

        assert_eq!(overridden.url(), None);
        assert_eq!(overridden.network(), Some(Network::Sepolia));
    }

    #[test]
    fn test_network_params_override_empty_keeps_base() {
        let base = NetworkParams::new(Some(Url::parse("https://base.example.com").unwrap()), None)
            .unwrap();
        let other = NetworkParams::default();
        let result = base.override_with(other);

        assert_eq!(result.url(), base.url());
        assert_eq!(result.network(), None);
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
