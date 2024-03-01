#[starknet::interface]
trait IReplaceBytecode<TContractState> {
    fn get(self: @TContractState) -> felt252;
}

#[starknet::contract]
mod ReplaceBytecodeA {
    #[storage]
    struct Storage {}

    #[abi(embed_v0)]
    impl IReplaceBytecodeA of super::IReplaceBytecode<ContractState> {
        fn get(self: @ContractState) -> felt252 {
            2137
        }
    }
}

#[starknet::contract]
mod ReplaceBytecodeB {
    #[storage]
    struct Storage {}

    #[abi(embed_v0)]
    impl IReplaceBytecodeB of super::IReplaceBytecode<ContractState> {
        fn get(self: @ContractState) -> felt252 {
            420
        }
    }
}
