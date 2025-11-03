#[starknet::interface]
pub trait IValueStorage<TContractState> {
    fn set_value(ref self: TContractState, value: u128);
    fn get_value(self: @TContractState) -> u128;
}

#[starknet::contract]
pub mod ValueStorage {
    use starknet::storage::StoragePointerWriteAccess;
    use starknet::storage::StoragePointerReadAccess;

#[storage]
    struct Storage {
        stored_value: u128,
    }

    #[abi(embed_v0)]
    impl ValueStorageImpl of super::IValueStorage<ContractState> {
        fn set_value(ref self: ContractState, value: u128) {
            self.stored_value.write(value);
        }

        fn get_value(self: @ContractState) -> u128 {
            self.stored_value.read()
        }
    }
}

pub fn add(a: felt252, b: felt252) -> felt252 {
    a + b
}

pub fn fib(a: felt252, b: felt252, n: felt252) -> felt252 {
    match n {
        0 => a,
        _ => fib(b, a + b, n - 1),
    }
}
