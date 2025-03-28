#[starknet::interface]
trait IReplaceBytecodeA<TContractState> {
    fn get(self: @TContractState) -> felt252;
    fn set(ref self: TContractState, value: felt252);
    fn get_const(self: @TContractState) -> felt252;
}

#[starknet::contract]
mod ReplaceBytecodeA {
    use starknet::storage::{StoragePointerReadAccess, StoragePointerWriteAccess};

    #[storage]
    struct Storage {
        value: felt252,
    }

    #[abi(embed_v0)]
    impl IReplaceBytecodeA of super::IReplaceBytecodeA<ContractState> {
        fn get(self: @ContractState) -> felt252 {
            self.value.read()
        }

        fn set(ref self: ContractState, value: felt252) {
            self.value.write(value);
        }

        fn get_const(self: @ContractState) -> felt252 {
            2137
        }
    }
}
