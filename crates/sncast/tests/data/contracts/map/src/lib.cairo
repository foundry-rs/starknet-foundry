#[starknet::interface]
trait IMap<TMapState> {
    fn put(ref self: TMapState, key: felt252, value: felt252);
    fn get(self: @TMapState, key: felt252) -> felt252;
}


#[starknet::contract]
mod Map {
    use starknet::{
        storage::{StoragePointerWriteAccess, StorageMapReadAccess, StoragePathEntry, Map},
    };
    #[storage]
    struct Storage {
        storage: Map<felt252, felt252>,
    }

    #[abi(embed_v0)]
    impl MapImpl of super::IMap<ContractState> {
        fn put(ref self: ContractState, key: felt252, value: felt252) {
            self.storage.entry(key).write(value);
        }

        fn get(self: @ContractState, key: felt252) -> felt252 {
            self.storage.read(key)
        }
    }
}
