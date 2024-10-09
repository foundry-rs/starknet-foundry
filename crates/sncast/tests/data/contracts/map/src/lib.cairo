#[starknet::interface]
trait IMap<TMapState> {
    fn put(ref self: TMapState, key: felt252, value: felt252);
    fn get(self: @TMapState, key: felt252) -> felt252;
}


#[starknet::contract]
mod Map {
    use starknet::storage::{Map as StarknetMap};
    #[storage]
    struct Storage {
        storage: StarknetMap::<felt252, felt252>,
    }

    #[abi(embed_v0)]
    impl Map of super::IMap<ContractState> {
        fn put(ref self: ContractState, key: felt252, value: felt252) {
            self.storage.write(key, value);
        }

        fn get(self: @ContractState, key: felt252) -> felt252 {
            self.storage.read(key)
        }
    }
}
