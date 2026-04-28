use starknet::ContractAddress;

#[starknet::interface]
pub trait Failing<TContractState> {
    fn fail(self: @TContractState, data: Array<felt252>);
}

#[starknet::interface]
pub trait Nested<TContractState> {
    fn nested(ref self: TContractState, contract_address: ContractAddress);
}

#[starknet::contract]
pub mod FailingContract {
    use super::Failing;

    #[storage]
    struct Storage {}

    // Note: missing #[abi(embed_v0)]
    impl FailingImpl of Failing<ContractState> {
        fn fail(self: @ContractState, data: Array<felt252>) {
            panic(data);
        }
    }
}

#[starknet::contract]
pub mod CallerContract {
    use starknet::ContractAddress;
    use super::{FailingSafeDispatcher, FailingSafeDispatcherTrait, Nested};

    #[storage]
    struct Storage {}

    #[abi(embed_v0)]
    impl NestedImpl of Nested<ContractState> {
        #[feature("safe_dispatcher")]
        fn nested(ref self: ContractState, contract_address: ContractAddress) {
            let dispatcher = FailingSafeDispatcher { contract_address };
            if let Ok(_) = dispatcher.fail(array![]) {
                panic!("should have panicked");
            }
        }
    }
}
