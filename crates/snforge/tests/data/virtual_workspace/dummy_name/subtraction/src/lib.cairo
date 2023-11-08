fn subtract(a: felt252, b: felt252) -> felt252 {
    a - b
}

#[starknet::contract]
mod SubtractionContract {
    use super::subtract;

    #[storage]
    struct Storage {}

    #[external(v0)]
    fn answer(ref self: ContractState) -> felt252 {
        subtract(10, 20)
    }
}

#[cfg(test)]
mod tests {
    use super::subtract;

    #[test]
    #[available_gas(100000)]
    fn it_works() {
        assert(subtract(3, 2) == 1, 'it works!');
    }
}
