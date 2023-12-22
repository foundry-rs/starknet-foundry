#[starknet::interface]
trait ISimpleContract<T> {
    fn simple_call(self: @T, data: felt252) -> felt252;
}

#[starknet::contract]
mod SimpleContract {
    #[storage]
    struct Storage {}

    #[abi(embed_v0)]
    impl ISimpleContractImpl of super::ISimpleContract<ContractState> {
        fn simple_call(self: @ContractState, data: felt252) -> felt252 {
            2 * data
        }
    }
}
