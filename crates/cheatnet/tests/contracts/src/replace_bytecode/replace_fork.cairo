#[starknet::interface]
trait IReplaceInFork<TContractState> {
    fn get_owner(self: @TContractState) -> felt252;
}

#[starknet::contract]
mod ReplaceInFork {
    #[storage]
    struct Storage {}

    #[abi(embed_v0)]
    impl IReplaceInFork of super::IReplaceInFork<ContractState> {
        fn get_owner(self: @ContractState) -> felt252 {
            0
        }
    }
}
