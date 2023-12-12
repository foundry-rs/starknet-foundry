#[starknet::contract]
mod MapComplexValueSimpleKey {
    #[derive(Serde, Drop, starknet::Store)]
    struct NestedStructure {
        c: felt252
    }
    #[derive(Serde, Drop, starknet::Store)]
    struct StoredStructure {
        a: felt252,
        b: NestedStructure,
    }


    #[storage]
    struct Storage {
        values: LegacyMap<felt252, StoredStructure>,
    }

    #[external(v0)]
    fn insert(ref self: ContractState, key: felt252, value: StoredStructure) {
        self.values.write(key, value);
    }

    #[external(v0)]
    fn read(self: @ContractState, key: felt252) -> StoredStructure {
        self.values.read(key)
    }
}
