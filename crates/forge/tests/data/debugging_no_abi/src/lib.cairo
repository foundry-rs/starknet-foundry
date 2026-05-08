#[starknet::interface]
pub trait External<TContractState> {
    fn call(ref self: TContractState);
}


#[starknet::interface]
pub trait FailingInternal<TContractState> {
    fn fail(self: @TContractState, data: Array<felt252>);
}

#[starknet::contract]
pub mod CallerContract {
    use super::{
        External, FailingInternal, FailingInternalSafeDispatcher,
        FailingInternalSafeDispatcherTrait,
    };

    #[storage]
    struct Storage {}

    #[abi(embed_v0)]
    impl ExternalImpl of External<ContractState> {
        #[feature("safe_dispatcher")]
        fn call(ref self: ContractState) {
            let dispatcher = FailingInternalSafeDispatcher {
                contract_address: starknet::get_contract_address(),
            };
            if let Ok(_) = dispatcher.fail(array![]) {
                panic!("should have panicked");
            }
        }
    }

    // Note: missing #[abi(embed_v0)]
    impl FailingInternalImpl of FailingInternal<ContractState> {
        fn fail(self: @ContractState, data: Array<felt252>) {
            panic(data);
        }
    }
}
