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
        default = "default_block_explorer",
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
}

/// Default block explorer function
fn default_block_explorer() -> Option<block_explorer::Service> {
    Some(block_explorer::Service::default())
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
        }
    }
}

impl Config for CastConfig {
    #[must_use]
    fn tool_name() -> &'static str {
        "sncast"
    }

    fn from_raw(config: serde_json::Value) -> Result<Self> {
        Ok(serde_json::from_value::<CastConfig>(config)?)
    }
}
