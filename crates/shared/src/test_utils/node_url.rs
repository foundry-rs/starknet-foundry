use anyhow::{Context, Result};
use std::env;
use url::Url;

/// Loads node RPC URL from `SEPOLIA_RPC_URL` environmental variable set in dotenv.
///
/// #### Note:
/// - `node_rpc_url()` -> <https://example.com/rpc/v0_7>
/// - `node_url()` -> <https://example.com/>
pub fn node_rpc_url() -> Result<Url> {
    Url::parse("http://188.34.188.184:7070/rpc/v0_7")
        .with_context(|| "Failed to parse the sepolia RPC URL")
}

/// Loads node URL from `SEPOLIA_RPC_URL` environmental variable and parses it,
/// returning URL with no slug (`rpc/v0_7` suffix).
pub fn node_url() -> Result<Url> {
    let mut node_url = node_rpc_url()?;
    node_url.set_path("");
    node_url.set_query(None);

    Ok(node_url)
}
