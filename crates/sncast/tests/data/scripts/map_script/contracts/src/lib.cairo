#[starknet::interface]
trait IMap<TMapState> {
    fn put(ref self: TMapState, key: felt252, value: felt252);
    fn get(self: @TMapState, key: felt252) -> felt252;
    fn dummy(self: @TMapState) -> felt252;
}


#[starknet::contract]
mod Mapa {
    #[storage]
    struct Storage {
        storage: LegacyMap::<felt252, felt252>,
    }

    #[abi(embed_v0)]
    impl Map of super::IMap<ContractState> {
        fn put(ref self: ContractState, key: felt252, value: felt252) {
            self.storage.write(key, value);
        }

        fn get(self: @ContractState, key: felt252) -> felt252 {
            self.storage.read(key)
        }

        fn dummy(self: @ContractState) -> felt252 {
            1
        }
    }
}

#[starknet::contract]
mod Mapa2 {
    #[storage]
    struct Storage {
        storage: LegacyMap::<felt252, felt252>,
    }

    #[abi(embed_v0)]
    impl Map of super::IMap<ContractState> {
        fn put(ref self: ContractState, key: felt252, value: felt252) {
            self.storage.write(key, value);
        }

        fn get(self: @ContractState, key: felt252) -> felt252 {
            self.storage.read(key)
        }

        fn dummy(self: @ContractState) -> felt252 {
            1
        }
    }
}

