#[starknet::contract]
mod StorageTester {
    use starknet::storage::Map;

    #[derive(Serde, Drop, starknet::Store)]
    struct NestedStructure {
        c: felt252
    }
    #[derive(Serde, Drop, starknet::Store)]
    struct StoredStructure {
        a: felt252,
        b: NestedStructure,
    }

    #[derive(Serde, Drop, starknet::Store, Hash)]
    struct NestedKey {
        c: felt252
    }
    #[derive(Serde, Drop, starknet::Store, Hash)]
    struct StructuredKey {
        a: felt252,
        b: NestedKey,
    }

    #[storage]
    struct Storage {
        structure: StoredStructure,
        felt_to_structure: Map<felt252, StoredStructure>,
        structure_to_felt: Map<StructuredKey, felt252>,
        felt_to_felt: Map<felt252, felt252>,
    }

    #[external(v0)]
    fn insert_structure(ref self: ContractState, value: StoredStructure) {
        self.structure.write(value);
    }

    #[external(v0)]
    fn read_structure(self: @ContractState) -> StoredStructure {
        self.structure.read()
    }

    #[external(v0)]
    fn insert_felt_to_structure(ref self: ContractState, key: felt252, value: StoredStructure) {
        self.felt_to_structure.write(key, value);
    }

    #[external(v0)]
    fn read_felt_to_structure(self: @ContractState, key: felt252) -> StoredStructure {
        self.felt_to_structure.read(key)
    }

    #[external(v0)]
    fn insert_structure_to_felt(ref self: ContractState, key: StructuredKey, value: felt252) {
        self.structure_to_felt.write(key, value);
    }

    #[external(v0)]
    fn read_structure_to_felt(self: @ContractState, key: StructuredKey) -> felt252 {
        self.structure_to_felt.read(key)
    }

    #[external(v0)]
    fn insert_felt_to_felt(ref self: ContractState, key: felt252, value: felt252) {
        self.felt_to_felt.write(key, value);
    }

    #[external(v0)]
    fn read_felt_to_felt(self: @ContractState, key: felt252) -> felt252 {
        self.felt_to_felt.read(key)
    }
}
