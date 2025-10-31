pub const EXPECTED_RPC_VERSION: &str = "0.9.0";
pub const RPC_URL_VERSION: &str = "v0_9";
pub const SNFORGE_TEST_FILTER: &str = "SNFORGE_TEST_FILTER";
const ALCHEMY_API_KEY: &str = env!("ALCHEMY_API_KEY");

#[must_use]
pub fn free_rpc_provider_url() -> String {
    format!(
        "https://starknet-sepolia.g.alchemy.com/starknet/version/rpc/{RPC_URL_VERSION}/{ALCHEMY_API_KEY}"
    )
}
