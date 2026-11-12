use crate::consts::SEPOLIA_RPC_URL;
use url::Url;

/// #### Note:
/// - `node_rpc_url()` -> <https://rpc.pathfinder.swmansion.com/testnet-sepolia/rpc/v0_10>
/// - `node_url()` -> <https://rpc.pathfinder.swmansion.com/testnet-sepolia/>
#[must_use]
pub fn node_rpc_url() -> Url {
    Url::parse(SEPOLIA_RPC_URL).expect("Failed to parse the sepolia RPC URL")
}

/// Returns the network base URL without the versioned RPC suffix.
#[must_use]
pub fn node_url() -> Url {
    node_rpc_url()
        .join("../")
        .expect("Failed to parse the node URL")
}
