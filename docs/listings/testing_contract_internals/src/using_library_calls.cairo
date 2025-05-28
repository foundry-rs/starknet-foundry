#[starknet::interface]
pub trait ILibraryContract<TContractState> {
    fn get_value(self: @TContractState) -> felt252;
    fn set_value(ref self: TContractState, number: felt252);
}

#[starknet::contract]
pub mod LibraryContract {
    use starknet::storage::{StoragePointerReadAccess, StoragePointerWriteAccess};
    #[storage]
    struct Storage {
        value: felt252,
    }

    #[external(v0)]
    pub fn get_value(self: @ContractState) -> felt252 {
        self.value.read()
    }

    #[external(v0)]
    pub fn set_value(ref self: ContractState, number: felt252) {
        self.value.write(number);
    }
}
