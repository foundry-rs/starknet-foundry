use starknet::macros::felt;
use starknet_types_core::felt::Felt;

pub const ACCOUNT: &str = "user1";
pub const ACCOUNT_FILE_PATH: &str = "tests/data/accounts/accounts.json";
pub const SEPOLIA_RPC_URL: &str = "http://188.34.188.184:7070/rpc/v0_7";

pub const URL: &str = "http://127.0.0.1:5055/rpc";
pub const NETWORK: &str = "testnet";
pub const SEED: u32 = 1_053_545_548;

// Block number used by devnet to fork the Sepolia testnet network in the tests
pub const FORK_BLOCK_NUMBER: u32 = 369_169;

pub const CONTRACTS_DIR: &str = "tests/data/contracts";
pub const SCRIPTS_DIR: &str = "tests/data/scripts";
pub const MULTICALL_CONFIGS_DIR: &str = "crates/sncast/tests/data/multicall_configs";

pub const DEVNET_OZ_CLASS_HASH_CAIRO_0: &str =
    "0x4d07e40e93398ed3c76981e72dd1fd22557a78ce36c0515f679e27f0bb5bc5f";
pub const DEVNET_PREDEPLOYED_ACCOUNT_ADDRESS: &str =
    "0x691a61b12a7105b1372cc377f135213c11e8400a546f6b0e7ea0296046690ce";

// OpenZeppelin account contract v0.8.1
pub const DEVNET_OZ_CLASS_HASH_CAIRO_1: Felt =
    felt!("0x061dac032f228abef9c6626f995015233097ae253a7f72d68552db02f2971b8f");

pub const MAP_CONTRACT_ADDRESS_SEPOLIA: &str =
    "0xcd8f9ab31324bb93251837e4efb4223ee195454f6304fcfcb277e277653008";

pub const MAP_CONTRACT_CLASS_HASH_SEPOLIA: &str =
    "0x2a09379665a749e609b4a8459c86fe954566a6beeaddd0950e43f6c700ed321";

pub const MAP_CONTRACT_DECLARE_TX_HASH_SEPOLIA: &str =
    "0x4f644d3ea723b9c28781f2bea76e9c2cd8cc667b2861faf66b4e45402ea221c";

pub const CONSTRUCTOR_WITH_PARAMS_CONTRACT_CLASS_HASH_SEPOLIA: &str =
    "0x59426c817fb8103edebdbf1712fa084c6744b2829db9c62d1ea4dce14ee6ded";

pub const DATA_TRANSFORMER_CONTRACT_ADDRESS_SEPOLIA: &str =
    "0x016ad425af4585102e139d4fb2c76ce786d1aaa1cfcd88a51f3ed66601b23cdd";

pub const MAP_CONTRACT_NAME: &str = "Map";
