#[starknet::contract]
mod StorageTester {
    use hash::LegacyHash;

    #[derive(Serde, Drop, starknet::Store)]
    struct NestedStructure {
        c: felt252
    }
    #[derive(Serde, Drop, starknet::Store)]
    struct StoredStructure {
        a: felt252,
        b: NestedStructure,
    }

    impl NestedKeyHash of LegacyHash<NestedKey> {
        fn hash(state: felt252, value: NestedKey) -> felt252 {
            LegacyHash::<felt252>::hash(state, value.c)
        }
    }

    impl StructuredKeyHash of LegacyHash<StructuredKey> {
        fn hash(state: felt252, value: StructuredKey) -> felt252 {
            let state = LegacyHash::<felt252>::hash(state, value.a);
            LegacyHash::<NestedKey>::hash(state, value.b)
        }
    }

    #[derive(Serde, Drop, starknet::Store, LegacyHash)]
    struct NestedKey {
        c: felt252
    }
    #[derive(Serde, Drop, starknet::Store, LegacyHash)]
    struct StructuredKey {
        a: felt252,
        b: NestedKey,
    }

    #[storage]
    struct Storage {
        structure: StoredStructure,
        felt_to_structure: LegacyMap<felt252, StoredStructure>,
        structure_to_felt: LegacyMap<StructuredKey, felt252>,
        felt_to_felt: LegacyMap<felt252, felt252>,
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
