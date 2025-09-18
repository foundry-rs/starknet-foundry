use url::Url;

/// #### Note:
/// - `node_rpc_url()` -> <https://example.com/rpc/v0_7>
/// - `node_url()` -> <https://example.com/>
#[must_use]
pub fn node_rpc_url() -> Url {
    Url::parse("http://188.34.188.184:7070/rpc/v0_9").expect("Failed to parse the sepolia RPC URL")
}

/// returning URL with no slug (`rpc/v0_7` suffix).
#[must_use]
pub fn node_url() -> Url {
    let mut node_url = node_rpc_url();
    node_url.set_path("");
    node_url.set_query(None);

    node_url
}
