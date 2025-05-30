use conversions::string::TryFromHexStr;
use starknet_api::core::ContractAddress;

use crate::predeployment::predeployed_contract::PredeployedContract;

use super::constructor_data::ERC20ConstructorData;

pub const STRK_CONTRACT_ADDRESS: &str =
    "0x4718f5a0fc34cc1af16a1cdee98ffb20c31f5cd61d6ab07201858f4287c938d";

#[must_use]
pub fn strk_predeployed_contract() -> PredeployedContract {
    // Compiled with starknet-compile, compiler version: 2.10.0
    // See: https://github.com/starknet-io/starkgate-contracts/blob/c787ec8e727c45499700d01e4eacd4cbc23a36ea/src/cairo/strk/erc20_lockable.cairo
    let raw_casm = include_str!("../../data/strk_erc20_casm.json");

    let contract_address = ContractAddress::try_from_hex_str(STRK_CONTRACT_ADDRESS).unwrap();
    let class_hash = TryFromHexStr::try_from_hex_str(
        "0x04ad3c1dc8413453db314497945b6903e1c766495a1e60492d44da9c2a986e4b",
    )
    .unwrap();

    // All storage values are taken from https://starkscan.co/contract/0x04718f5a0fc34cc1af16a1cdee98ffb20c31f5cd61d6ab07201858f4287c938d#contract-storage
    // Block 747469
    let total_supply_low: u128 = 60_000_000_000_000_000_000_000_000;
    let permitted_minter = ContractAddress::try_from_hex_str(
        "0x594c1582459ea03f77deaf9eb7e3917d6994a03c13405ba42867f83d85f085d",
    )
    .unwrap();

    let constructor_data = ERC20ConstructorData {
        name: "STRK".to_string(),
        symbol: "STRK".to_string(),
        decimals: 18,
        total_supply: (total_supply_low, 0),
        permitted_minter,
        upgrade_delay: 0,
    };

    PredeployedContract::erc20(contract_address, class_hash, raw_casm, constructor_data)
}
