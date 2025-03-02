#[starknet::interface]
trait IReplaceBytecodeB<TContractState> {
    fn get(self: @TContractState) -> felt252;
    fn set(ref self: TContractState, value: felt252);
    fn get_const(self: @TContractState) -> felt252;
}

#[starknet::contract]
mod ReplaceBytecodeB {
    use core::starknet::storage::{StoragePointerReadAccess, StoragePointerWriteAccess};

    #[storage]
    struct Storage {
        value: felt252,
    }

    #[abi(embed_v0)]
    impl IReplaceBytecodeB of super::IReplaceBytecodeB<ContractState> {
        fn get(self: @ContractState) -> felt252 {
            self.value.read() + 100
        }

        fn set(ref self: ContractState, value: felt252) {
            self.value.write(value + 100);
        }

        fn get_const(self: @ContractState) -> felt252 {
            420
        }
    }
}
