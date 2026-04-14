use conversions::string::TryFromHexStr;

use crate::predeployment::predeployed_contract::PredeployedContract;

use super::constructor_data::ERC20ConstructorData;

pub const ETH_CONTRACT_ADDRESS: &str =
    "0x49d36570d4e46f48e99674bd3fcc84644ddd6b96f7c741b1562b82f9e004dc7";
pub const ETH_SIERRA_CLASS_HASH: &str =
    "0x00b45dbc3714180381c5680e41931172d67194d77d504413465390e0bef194ec";

#[must_use]
pub fn eth_predeployed_contract() -> PredeployedContract {
    // starkgate-contracts v3.0.0
    // Link to Cairo contract: https://github.com/starknet-io/starkgate-contracts/blob/07e11c39119a10d5742735be5b1d51894ebf5311/packages/sg_token/src/erc20_mintable.cairo
    let raw_casm = include_str!("../../data/sg_token_ERC20Mintable.compiled_contract_class.json");

    let contract_address = TryFromHexStr::try_from_hex_str(ETH_CONTRACT_ADDRESS).unwrap();
    let class_hash = TryFromHexStr::try_from_hex_str(ETH_SIERRA_CLASS_HASH).unwrap();

    // All storage values are taken from https://voyager.online/contract/0x049d36570d4e46f48e99674bd3fcc84644ddd6b96f7c741b1562b82f9e004dc7#readStorage
    // Block 747469
    let total_supply_low: u128 = 15_000_000_000_000_000_000_000;
    let permitted_minter = TryFromHexStr::try_from_hex_str(
        "0x4c5772d1914fe6ce891b64eb35bf3522aeae1315647314aac58b01137607f3f",
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
