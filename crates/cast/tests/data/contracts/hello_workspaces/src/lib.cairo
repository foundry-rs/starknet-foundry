#[starknet::contract]
mod FibContract {
    use addition::add;
    use fibonacci::fib;

    #[storage]
    struct Storage {}

    #[external(v0)]
    fn answer(ref self: ContractState) -> felt252 {
        add(fib(0, 1, 16), fib(0, 1, 8))
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_simple() {
        assert(1 == 1, 1);
    }
}
