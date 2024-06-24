use starknet::ContractAddress;

// 0x2c77ca97586968c6651a533bd5f58042c368b14cf5f526d2f42f670012e10ac
#[starknet::interface]
trait ICairo0Contract<TContractState> {
    // this function only job is to emit `my_event` with single felt252 value
    fn emit_one_cairo0_event(ref self: TContractState, contract_address: felt252);
}

#[starknet::interface]
trait ISpyEventsCairo0<TContractState> {
    fn test_cairo0_event_collection(ref self: TContractState, cairo0_address: ContractAddress);
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
        fn test_cairo0_event_collection(ref self: ContractState, cairo0_address: ContractAddress) {
            let cairo0_contract = super::ICairo0ContractDispatcher {
                contract_address: cairo0_address
            };

            cairo0_contract.emit_one_cairo0_event(123456789);
        }
    }
}
