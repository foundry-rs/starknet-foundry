#[starknet::contract]
mod MapSimpleValueComplexKey {
    use hash::LegacyHash;

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
        values: LegacyMap<StructuredKey, felt252>,
    }

    #[external(v0)]
    fn insert(ref self: ContractState, key: StructuredKey, value: felt252) {
        self.values.write(key, value);
    }

    #[external(v0)]
    fn read(self: @ContractState, key: StructuredKey) -> felt252 {
        self.values.read(key)
    }
}
