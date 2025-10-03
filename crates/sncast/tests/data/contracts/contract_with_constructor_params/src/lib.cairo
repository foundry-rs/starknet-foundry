#[starknet::contract]
mod ContractWithConstructorParams {
    #[storage]
    struct Storage {}

    #[constructor]
    fn constructor(ref self: ContractState, foo: felt252, bar: felt252) {
        let _x = foo + bar;
    }
}
