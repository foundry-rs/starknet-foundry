#[starknet::contract]
mod MapSimpleValueSimpleKey {
    use starknet::storage::{Map, StorageMapReadAccess, StoragePathEntry, StoragePointerWriteAccess};
    #[storage]
    struct Storage {
        values: Map<felt252, felt252>,
    }

    #[external(v0)]
    fn insert(ref self: ContractState, key: felt252, value: felt252) {
        self.values.entry(key).write(value);
    }

    #[external(v0)]
    fn read(self: @ContractState, key: felt252) -> felt252 {
        self.values.read(key)
    }
}
