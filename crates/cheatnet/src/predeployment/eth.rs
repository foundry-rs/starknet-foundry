use conversions::string::TryFromHexStr;
use starknet_api::core::ContractAddress;

use super::predeployed_contract::{ERC20ConstructorData, PredeployedContract};

pub const ETH_CONTRACT_ADDRESS: &str =
    "0x49d36570d4e46f48e99674bd3fcc84644ddd6b96f7c741b1562b82f9e004dc7";

#[must_use]
pub fn eth_predeployed_contract() -> PredeployedContract {
    // Compiled with starknet-compile, compiler version: 2.10.0
    // Fetched with `starknet_getCompiledCasm`
    let raw_casm = include_str!("../data/eth_erc20_casm.json");

    let contract_address = ContractAddress::try_from_hex_str(ETH_CONTRACT_ADDRESS).unwrap();
    let class_hash = TryFromHexStr::try_from_hex_str(
        "0x076791ef97c042f81fbf352ad95f39a22554ee8d7927b2ce3c681f3418b5206a",
    )
    .unwrap();

    // All storage values are taken from https://starkscan.co/contract/0x04718f5a0fc34cc1af16a1cdee98ffb20c31f5cd61d6ab07201858f4287c938d#contract-storage
    let total_supply_low: u128 = 15_000_000_000_000_000_000_000;
    let permitted_minter = ContractAddress::try_from_hex_str(
        "0x4C5772D1914FE6CE891B64EB35BF3522AEAE1315647314AAC58B01137607F3F",
    )
    .unwrap();

    let constructor_data = ERC20ConstructorData {
        name: "ETH".to_string(),
        symbol: "ETH".to_string(),
        decimals: 18,
        total_supply: (total_supply_low, 0),
        permitted_minter,
        upgrade_delay: 0,
    };

    PredeployedContract::erc20(contract_address, class_hash, raw_casm, constructor_data)
}
