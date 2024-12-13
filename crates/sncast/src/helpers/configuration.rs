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
#[non_exhaustive]
pub struct CastConfigFromFile {
    /// RPC url
    url: Option<String>,

    account: Option<String>,

    #[serde(rename(serialize = "accounts-file", deserialize = "accounts-file"))]
    accounts_file: Option<Utf8PathBuf>,

    keystore: Option<Utf8PathBuf>,

    #[serde(rename(serialize = "wait-params", deserialize = "wait-params"))]
    wait_params: Option<ValidatedWaitParams>,

    #[serde(rename(serialize = "block-explorer", deserialize = "block-explorer"))]
    /// A block explorer service, used to display links to transaction details
    block_explorer: Option<block_explorer::Service>,

    #[serde(rename(serialize = "show-explorer-links", deserialize = "show-explorer-links"))]
    /// Print links pointing to pages with transaction details in the chosen block explorer
    show_explorer_links: Option<bool>,
}

impl CastConfigFromFile {
    #[must_use]
    pub fn url(&self) -> &Option<String> {
        &self.url
    }

    #[must_use]
    pub fn account(&self) -> &Option<String> {
        &self.account
    }

    #[must_use]
    pub fn accounts_file(&self) -> &Option<Utf8PathBuf> {
        &self.accounts_file
    }

    #[must_use]
    pub fn keystore(&self) -> &Option<Utf8PathBuf> {
        &self.keystore
    }

    #[must_use]
    pub fn wait_params(&self) -> Option<ValidatedWaitParams> {
        self.wait_params
    }

    #[must_use]
    pub fn block_explorer(&self) -> Option<block_explorer::Service> {
        self.block_explorer
    }

    #[must_use]
    pub fn show_explorer_links(&self) -> Option<bool> {
        self.show_explorer_links
    }

    #[must_use]
    pub fn combine(self, other: Self) -> Self {
        Self {
            url: self.url.if_some_or(other.url),
            account: self.account.if_some_or(other.account),
            accounts_file: self.accounts_file.if_some_or(other.accounts_file),
            keystore: self.keystore.if_some_or(other.keystore),
            wait_params: self.wait_params.if_some_or(other.wait_params),
            block_explorer: self.block_explorer.if_some_or(other.block_explorer),
            show_explorer_links: self.show_explorer_links.or(other.show_explorer_links),
        }
    }
}

pub trait IfSomeOr<T> {
    fn if_some_or(self, b: Option<T>) -> Option<T>;
}

impl<T> IfSomeOr<T> for Option<T> {
    fn if_some_or(self, b: Option<T>) -> Option<T> {
        match (self, b) {
            (Some(a), _) => Some(a),
            (None, Some(b)) => Some(b),
            (None, None) => None,
        }
    }
}

// fn combine_options<T>(a: Option<T>, b: Option<T>) -> Option<T> {
//     match (a, b) {
//         (Some(a), Some(b)) => Some(a),
//
//     }
// }

impl Config for CastConfigFromFile {
    #[must_use]
    fn tool_name() -> &'static str {
        "sncast"
    }

    fn from_raw(config: serde_json::Value) -> Result<Self> {
        Ok(serde_json::from_value::<CastConfigFromFile>(config)?)
    }
}
