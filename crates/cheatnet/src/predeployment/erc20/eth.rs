use conversions::string::TryFromHexStr;
use starknet_api::core::ContractAddress;

use crate::predeployment::{load_gzipped_artifact, predeployed_contract::PredeployedContract};

use super::constructor_data::ERC20ConstructorData;

pub const ETH_CONTRACT_NAME: &str = "ETH Token";
pub const ETH_TOKEN_NAME: &str = "ETH";
pub const ETH_CONTRACT_ADDRESS: &str =
    "0x49d36570d4e46f48e99674bd3fcc84644ddd6b96f7c741b1562b82f9e004dc7";
pub const ERC20MINTABLE_SIERRA_CLASS_HASH: &str =
    "0x00b45dbc3714180381c5680e41931172d67194d77d504413465390e0bef194ec";

#[must_use]
pub fn eth_predeployed_contract() -> PredeployedContract {
    // starkgate-contracts v3.0.0
    // Link to Cairo contract: https://github.com/starknet-io/starkgate-contracts/blob/07e11c39119a10d5742735be5b1d51894ebf5311/packages/sg_token/src/erc20_mintable.cairo
    let raw_casm = load_gzipped_artifact(include_bytes!(
        "../../data/predeployed_contracts/ERC20Mintable/casm.json.gz"
    ))
    .expect("predeployed ETH CASM should be a valid gzip artifact");

    let contract_address = ContractAddress::try_from_hex_str(ETH_CONTRACT_ADDRESS).unwrap();
    let class_hash = TryFromHexStr::try_from_hex_str(ERC20MINTABLE_SIERRA_CLASS_HASH).unwrap();

    // All storage values are taken from https://voyager.online/contract/0x049d36570d4e46f48e99674bd3fcc84644ddd6b96f7c741b1562b82f9e004dc7#readStorage
    // Block 747469
    let total_supply_low: u128 = 15_000_000_000_000_000_000_000;
    let permitted_minter = ContractAddress::try_from_hex_str(
        "0x73314940630fd6dcda0d772d4c972c4e0a9946bef9dabf4ef84eda8ef542b82",
    )
    .unwrap();

    let constructor_data = ERC20ConstructorData {
        name: ETH_CONTRACT_NAME.to_string(),
        symbol: ETH_TOKEN_NAME.to_string(),
        decimals: 18,
        total_supply: (total_supply_low, 0),
        permitted_minter,
        upgrade_delay: 0,
    };

    PredeployedContract::erc20(contract_address, class_hash, &raw_casm, constructor_data)
}
