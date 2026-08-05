use starknet_rust::macros::felt;
use starknet_types_core::felt::Felt;

pub const ACCOUNT: &str = "user1";
pub const ACCOUNT_FILE_PATH: &str = "tests/data/accounts/accounts.json";

pub const SEPOLIA_RPC_URL: &str = "http://188.34.188.184:7070/rpc/v0_10";

pub const URL: &str = "http://127.0.0.1:5055/rpc";
pub const NETWORK: &str = "testnet";
pub const DEVNET_SEED: u32 = 1_053_545_548;
pub const DEVNET_ACCOUNTS_NUMBER: u8 = 20;

// Block number used by devnet to fork the Sepolia testnet network in the tests
pub const DEVNET_FORK_BLOCK_NUMBER: u32 = 7_776_133;

pub const CONTRACTS_DIR: &str = "tests/data/contracts";
pub const MULTICALL_CONFIGS_DIR: &str = "crates/sncast/tests/data/multicall_configs";

pub const DEVNET_OZ_CLASS_HASH_CAIRO_0: &str =
    "0x4d07e40e93398ed3c76981e72dd1fd22557a78ce36c0515f679e27f0bb5bc5f";
pub const DEVNET_PREDEPLOYED_ACCOUNT_ADDRESS: &str =
    "0x691a61b12a7105b1372cc377f135213c11e8400a546f6b0e7ea0296046690ce";

pub const DEVNET_OZ_CLASS_HASH_CAIRO_1: Felt =
    felt!("0x05b4b537eaa2399e3aa99c4e2e0208ebd6c71bc1467938cd52c798c601e43564");

pub use sncast::helpers::constants::{
    MAP_CONTRACT_ADDRESS_SEPOLIA, MAP_CONTRACT_CLASS_HASH_SEPOLIA,
    MAP_CONTRACT_DECLARE_TX_HASH_SEPOLIA,
};

pub const SUCCEEDED_TX_HASH: &str =
    "0x4cba686fa76bfa4b4ac788bf2ca9bfac3dd354561f2621c2ac7cf17fa46f75a";

pub const REVERTED_TX_HASH: &str =
    "0x00ae35dacba17cde62b8ceb12e3b18f4ab6e103fa2d5e3d9821cb9dc59d59a3c";

pub const CONSTRUCTOR_WITH_PARAMS_CONTRACT_CLASS_HASH_SEPOLIA: &str =
    "0x59426c817fb8103edebdbf1712fa084c6744b2829db9c62d1ea4dce14ee6ded";

pub const DATA_TRANSFORMER_CONTRACT_ADDRESS_SEPOLIA: &str =
    "0x00351c816183324878714973f3da1a43c1a40d661b8dac5cb69294cc333342ed";
pub const DATA_TRANSFORMER_CONTRACT_CLASS_HASH_SEPOLIA: &str =
    "0x0786d1f010d66f838837290e472415186ba6a789fb446e7f92e444bed7b5d9c0";
pub const DATA_TRANSFORMER_CONTRACT_ABI_PATH: &str =
    "tests/data/files/data_transformer_contract_abi.json";
pub const DATA_TRANSFORMER_CONTRACT_DIR: &str =
    "../../crates/data-transformer/tests/data/data_transformer";
