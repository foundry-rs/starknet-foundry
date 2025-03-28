#[starknet::interface]
pub trait ISimpleStorageContract<TState> {
    fn get_value(self: @TState, key: felt252) -> felt252;
}

#[starknet::contract]
pub mod SimpleStorageContract {
    use starknet::storage::Map;
    use starknet::storage::{StoragePointerWriteAccess, StorageMapReadAccess, StorageMapWriteAccess};

    #[storage]
    struct Storage {
        plain_felt: felt252,
        mapping: Map<felt252, felt252>,
    }

    #[constructor]
    fn constructor(ref self: ContractState) {
        self.plain_felt.write(0x2137_felt252);
        self.mapping.write('some_key', 'some_value');
    }

    #[external(v0)]
    pub fn get_value(self: @ContractState, key: felt252) -> felt252 {
        self.mapping.read(key)
    }
}
