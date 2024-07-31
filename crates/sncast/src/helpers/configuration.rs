use crate::ValidatedWaitParams;
use anyhow::Result;
use camino::Utf8PathBuf;
use configuration::GlobalConfig;
use serde::{Deserialize, Serialize};

#[derive(Default, Deserialize, Serialize, Clone, Debug, PartialEq)]
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
        rename(
            serialize = "block-explorer-search-url",
            deserialize = "block-explorer-search-url"
        )
    )]
    /// URL to an endpoint allowing contract address/class hash search in a block explorer
    pub block_explorer_search_url: Option<String>,
}

impl GlobalConfig for CastConfig {
    #[must_use]
    fn tool_name() -> &'static str {
        "sncast"
    }

    fn from_raw(config: serde_json::Value) -> Result<Self> {
        Ok(serde_json::from_value::<CastConfig>(config)?)
    }
}
