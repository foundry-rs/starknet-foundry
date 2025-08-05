#[starknet::interface]
trait IState<TState> {
    fn put(ref self: TState, key: felt252, value: felt252);
    fn get(self: @TState, key: felt252) -> felt252;
    fn dummy(self: @TState) -> felt252;
}


#[starknet::contract]
mod State {
    use starknet::storage::{Map, StorageMapReadAccess, StoragePathEntry, StoragePointerWriteAccess};

    #[storage]
    struct Storage {
        storage: Map<felt252, felt252>,
    }

    #[abi(embed_v0)]
    impl State of super::IState<ContractState> {
        fn put(ref self: ContractState, key: felt252, value: felt252) {
            self.storage.entry(key).write(value);
        }

        fn get(self: @ContractState, key: felt252) -> felt252 {
            self.storage.read(key)
        }

        fn dummy(self: @ContractState) -> felt252 {
            1
        }
    }
}
