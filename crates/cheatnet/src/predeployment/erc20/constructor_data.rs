use starknet_api::core::ContractAddress;

pub struct ERC20ConstructorData {
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub total_supply: (u128, u128), // (low, high)
    pub permitted_minter: ContractAddress,
    pub upgrade_delay: u64,
}
