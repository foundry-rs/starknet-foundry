pub fn add(a: felt252, b: felt252) -> felt252 {
    a + b
}

#[starknet::contract]
mod AdditionContract {
    use cast_addition::add;

    #[storage]
    struct Storage {}

    #[external(v0)]
    fn answer(ref self: ContractState) -> felt252 {
        add(10, 20)
    }
}
