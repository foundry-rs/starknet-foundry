use starknet::ContractAddress;

#[starknet::interface]
pub trait ICheatAccountContractAddressChecker<TContractState> {
    fn get_account_contract_address(ref self: TContractState) -> ContractAddress;
}

#[starknet::contract]
mod CheatAccountContractAddressChecker {
    use starknet::{ContractAddress, get_tx_info};

    #[storage]
    struct Storage {}

    #[abi(embed_v0)]
    impl CheatAccountAddressChecker of super::ICheatAccountContractAddressChecker<ContractState> {
        fn get_account_contract_address(ref self: ContractState) -> ContractAddress {
            let tx_info = get_tx_info();
            tx_info.account_contract_address
        }
    }
}
