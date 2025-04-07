#[starknet::interface]
pub trait IContract<TContractState> {
    fn response(ref self: TContractState) -> felt252;
}

#[cfg(feature: 'snforge_test_only')]
#[starknet::contract]
pub mod MockContract {
    #[storage]
    struct Storage {}

    #[abi(embed_v0)]
    impl IContractImpl of super::IContract<ContractState> {
        fn response(ref self: ContractState) -> felt252 {
            super::some_func()
        }
    }
}

fn some_func() -> felt252 {
    1234
}
