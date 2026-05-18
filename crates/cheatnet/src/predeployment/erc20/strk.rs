use conversions::string::TryFromHexStr;
use starknet_api::core::ContractAddress;

use crate::predeployment::{load_gzipped_artifact, predeployed_contract::PredeployedContract};

use super::constructor_data::ERC20ConstructorData;

pub const STRK_CONTRACT_NAME: &str = "STRK Token";
pub const STRK_TOKEN_NAME: &str = "STRK";
pub const STRK_CONTRACT_ADDRESS: &str =
    "0x4718f5a0fc34cc1af16a1cdee98ffb20c31f5cd61d6ab07201858f4287c938d";
pub const ERC20LOCKABLE_SIERRA_CLASS_HASH: &str =
    "0x02e77ee61d4df3d988ee1f42ea5442e913862cc82c2584d212ecda76666498fc";

#[must_use]
pub fn strk_predeployed_contract() -> PredeployedContract {
    // starkgate-contracts v3.0.0
    // Link to Cairo contract: https://github.com/starknet-io/starkgate-contracts/blob/07e11c39119a10d5742735be5b1d51894ebf5311/packages/strk/src/erc20_lockable.cairo
    let raw_casm = load_gzipped_artifact(include_bytes!(
        "../../data/predeployed_contracts/ERC20Lockable/casm.json.gz"
    ))
    .expect("predeployed STRK CASM should be a valid gzip artifact");

    let contract_address = ContractAddress::try_from_hex_str(STRK_CONTRACT_ADDRESS).unwrap();
    let class_hash = TryFromHexStr::try_from_hex_str(ERC20LOCKABLE_SIERRA_CLASS_HASH).unwrap();

    // All storage values are taken from https://voyager.online/contract/0x04718f5a0fc34cc1af16a1cdee98ffb20c31f5cd61d6ab07201858f4287c938d#readStorage
    // Block 747469
    let total_supply_low: u128 = 60_000_000_000_000_000_000_000_000;
    let permitted_minter = ContractAddress::try_from_hex_str(
        "0x594c1582459ea03f77deaf9eb7e3917d6994a03c13405ba42867f83d85f085d",
    )
    .unwrap();

    let constructor_data = ERC20ConstructorData {
        name: STRK_CONTRACT_NAME.to_string(),
        symbol: STRK_TOKEN_NAME.to_string(),
        decimals: 18,
        total_supply: (total_supply_low, 0),
        permitted_minter,
        upgrade_delay: 0,
    };

    PredeployedContract::erc20(contract_address, class_hash, &raw_casm, constructor_data)
}
