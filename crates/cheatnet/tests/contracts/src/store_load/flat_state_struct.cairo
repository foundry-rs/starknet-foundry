#[starknet::contract]
mod FlatStateStruct {
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
        value: StoredStructure,
    }

    #[external(v0)]
    fn insert(ref self: ContractState, value: StoredStructure) {
        self.value.write(value);
    }

    #[external(v0)]
    fn read(self: @ContractState) -> StoredStructure {
        self.value.read()
    }
}
