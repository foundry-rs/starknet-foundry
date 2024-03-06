use starknet::ContractAddress;

// https://testnet.starkscan.co/contract/0x1960625ba5c435bac113ecd15af3c60e327d550fc5dbb43f07cd0875ad2f54c
#[starknet::interface]
trait ICairo0Contract<TContractState> {
    // this function only job is to emit `my_event` with single felt252 value
    fn emit_one_cairo0_event(ref self: TContractState, contract_address: felt252);
}

#[starknet::interface]
trait ISpyEventsCairo0<TContractState> {
    fn test(ref self: TContractState, cairo0_address: ContractAddress);
}

#[starknet::contract]
mod SpyEventsCairo0 {
    use core::traits::Into;
    use starknet::{get_contract_address, get_caller_address, ContractAddress};
    use super::ICairo0ContractDispatcherTrait;

    #[storage]
    struct Storage {}

    #[abi(embed_v0)]
    impl ISpyEventsCairo0 of super::ISpyEventsCairo0<ContractState> {
        fn test(ref self: ContractState, cairo0_address: ContractAddress) {
            let cairo0_contract = super::ICairo0ContractDispatcher {
                contract_address: cairo0_address
            };

            cairo0_contract.emit_one_cairo0_event(123456789);
        }
    }
}
