fn add(a: felt252, b: felt252) -> felt252 {
    a + b
}

#[starknet::contract]
mod AdditionContract {
    use addition::add;

    #[storage]
    struct Storage {}

    #[abi(embed_v0)]
    fn answer(ref self: ContractState) -> felt252 {
        add(10, 20)
    }
}

#[cfg(test)]
mod tests {
    use super::add;

    #[test]
    fn it_works() {
        assert(add(2, 3) == 5, 'it works!');
    }
}
