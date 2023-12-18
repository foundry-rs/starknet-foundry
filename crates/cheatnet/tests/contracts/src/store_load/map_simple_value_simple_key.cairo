#[starknet::contract]
mod MapSimpleValueSimpleKey {
    #[storage]
    struct Storage {
        values: LegacyMap<felt252, felt252>,
    }

    #[external(v0)]
    fn insert(ref self: ContractState, key: felt252, value: felt252) {
        self.values.write(key, value);
    }

    #[external(v0)]
    fn read(self: @ContractState, key: felt252) -> felt252 {
        self.values.read(key)
    }
}
