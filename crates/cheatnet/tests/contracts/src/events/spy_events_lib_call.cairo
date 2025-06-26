use starknet::{ClassHash, ContractAddress};

#[starknet::interface]
trait ISpyEventsLibCall<TContractState> {
    fn call_lib_call(ref self: TContractState, data: felt252, class_hash: ClassHash);
}

#[starknet::contract]
mod SpyEventsLibCall {
    use starknet::{ClassHash, ContractAddress};

    #[starknet::interface]
    trait ISpyEventsChecker<TContractState> {
        fn emit_one_event(ref self: TContractState, some_data: felt252);
    }

    #[storage]
    struct Storage {}

    #[abi(embed_v0)]
    impl ISpyEventsLibCallImpl of super::ISpyEventsLibCall<ContractState> {
        fn call_lib_call(ref self: ContractState, data: felt252, class_hash: ClassHash) {
            let spy_events_checker = ISpyEventsCheckerLibraryDispatcher { class_hash };
            spy_events_checker.emit_one_event(data);
        }
    }
}
