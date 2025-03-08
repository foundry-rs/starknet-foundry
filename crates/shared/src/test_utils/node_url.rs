use url::Url;

/// Loads node RPC URL from `SEPOLIA_RPC_URL` environmental variable set in dotenv.
///
/// #### Note:
/// - `node_rpc_url()` -> <https://example.com/rpc/v0_7>
/// - `node_url()` -> <https://example.com/>
#[must_use]
pub fn node_rpc_url() -> Url {
    // FIXME
    // Url::parse("http://188.34.188.184:7070/rpc/v0_7").expect("Failed to parse the sepolia RPC URL")
    Url::parse("https://rpc.pathfinder.equilibrium.co/testnet-sepolia/rpc/v0_8")
        .expect("Failed to parse the sepolia RPC URL")
}

/// Loads node URL from `SEPOLIA_RPC_URL` environmental variable and parses it,
/// returning URL with no slug (`rpc/v0_7` suffix).
#[must_use]
pub fn node_url() -> Url {
    let mut node_url = node_rpc_url();
    node_url.set_path("");
    node_url.set_query(None);

    node_url
}
