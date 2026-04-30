#[starknet::contract]
mod ContractWithConstructorParams {
    use starknet::storage::StoragePointerWriteAccess;

    #[storage]
    struct Storage {
        salt: felt252,
    }

    #[constructor]
    fn constructor(ref self: ContractState, foo: felt252, bar: felt252) {
        self.salt.write(foo + bar);
    }
}
