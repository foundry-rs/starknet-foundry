#[starknet::interface]
pub trait ISimpleStorageContract<TState> {}

#[starknet::contract]
mod SimpleStorageContract {
    use starknet::storage::Map;

    #[storage]
    struct Storage {
        plain_felt: felt252,
        mapping: Map<felt252, felt252>,
    }

    #[constructor]
    fn constructor(ref self: ContractState) {
        self.plain_felt.write(0x2137_felt252);
        self.mapping.write('some_key', 'some_value');
    }
}
