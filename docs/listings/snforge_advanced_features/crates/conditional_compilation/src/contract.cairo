#[starknet::interface]
pub trait IMockContract<TContractState> {
    fn response(self: @TContractState) -> u32;
}

#[starknet::contract]
#[cfg(feature: 'enable_for_tests')]
mod MockContract {
    #[storage]
    struct Storage {}

    #[abi(embed_v0)]
    impl IMockContractImpl of super::IMockContract<ContractState> {
        fn response(self: @ContractState) -> u32 {
            1
        }
    }
}
