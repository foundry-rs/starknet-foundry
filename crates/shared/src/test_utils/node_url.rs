use anyhow::{Context, Result};
use std::env;
use url::Url;

/// Loads node RPC URL from `SEPOLIA_RPC_URL` environmental variable set in dotenv.
///
/// #### Note:
/// - `node_rpc_url()` -> <https://example.com/rpc/v0_7>
/// - `node_url()` -> <https://example.com/>
pub fn node_rpc_url() -> Result<Url> {
    dotenv::dotenv().ok();
    let node_rpc_url = env::var("SEPOLIA_RPC_URL")
        .context("The required environmental variable `SEPOLIA_RPC_URL` is not set. Please set it manually or in .env file"
    )?;

    Url::parse(&node_rpc_url).with_context(|| {
        format!("Failed to parse the URL from the `SEPOLIA_RPC_URL` environmental variable: {node_rpc_url}")
    })
}

/// Loads node URL from `SEPOLIA_RPC_URL` environmental variable and parses it,
/// returning URL with no slug (`rpc/v0_7` suffix).
pub fn node_url() -> Result<Url> {
    let mut node_url = node_rpc_url()?;
    node_url.set_path("");
    node_url.set_query(None);

    Ok(node_url)
}
