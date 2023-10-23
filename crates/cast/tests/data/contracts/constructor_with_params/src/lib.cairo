#[starknet::contract]
mod ConstructorWithParams {
    #[storage]
    struct Storage {
        value1: felt252,
        value2: u256,
    }

    #[constructor]
    fn constructor(ref self: ContractState, first: felt252, second: u256) {
        self.value1.write(first);
        self.value2.write(second);
    }
}
