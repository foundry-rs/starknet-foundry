// TODO(#3087): EXPECTED_RPC_VERSION should be set to 0.8.0 once we use stable starknet-devnet release
// This is temporary adjustment so that the expected version will be 0.8.0-rc3 for the tests.
#[cfg(test)]
pub const EXPECTED_RPC_VERSION: &str = "0.8.0-rc3";
#[cfg(not(test))]
pub const EXPECTED_RPC_VERSION: &str = "0.8.0";

pub const RPC_URL_VERSION: &str = "v0_8";
pub const SNFORGE_TEST_FILTER: &str = "SNFORGE_TEST_FILTER";
pub const FREE_RPC_PROVIDER_URL: &str = "https://free-rpc.nethermind.io/sepolia-juno/v0_8";
