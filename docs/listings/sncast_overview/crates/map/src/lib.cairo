#[starknet::interface]
pub trait IMapContract<State> {
    fn put(ref self: State, key: felt252, value: felt252);
    fn get(self: @State, key: felt252) -> felt252;
}

#[starknet::contract]
pub mod MapContract {
    use starknet::storage::{Map, StorageMapReadAccess, StorageMapWriteAccess};

    #[storage]
    struct Storage {
        storage: Map<felt252, felt252>,
    }

    #[abi(embed_v0)]
    impl MapContractImpl of super::IMapContract<ContractState> {
        fn put(ref self: ContractState, key: felt252, value: felt252) {
            self.storage.write(key, value);
        }

        fn get(self: @ContractState, key: felt252) -> felt252 {
            self.storage.read(key)
        }
    }
}
