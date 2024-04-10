use crate::consts::RPC_URL_VERSION;
use anyhow::{Context, Result};
use std::env;
use url::Url;

pub mod output_assert;

pub fn rpc_url() -> Result<Url> {
    let rpc_url = env::var("RPC_URL")
        .with_context(|| "The required environmental variable `RPC_URL` is not set. Please set it manually or in .cargo/config.toml file"
    )?;

    Url::parse(&rpc_url).with_context(|| {
        format!("Failed to parse the URL from the `RPC_URL` environmental variable: {rpc_url}")
    })
}

pub fn rpc_url_with_version() -> Result<Url> {
    let mut url = rpc_url().unwrap();
    url.set_path(format!("rpc/{RPC_URL_VERSION}").as_str());
    url.set_query(None);

    Ok(url)
}
