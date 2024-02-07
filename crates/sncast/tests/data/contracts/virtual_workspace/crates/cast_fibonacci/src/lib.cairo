use cast_addition::add;

fn fib(a: felt252, b: felt252, n: felt252) -> felt252 {
    match n {
        0 => a,
        _ => fib(b, add(a, b), n - 1),
    }
}

#[starknet::contract]
mod FibonacciContract {
    use cast_addition::add;
    use cast_fibonacci::fib;

    #[storage]
    struct Storage {}

    #[external(v0)]
    fn answer(ref self: ContractState) -> felt252 {
        add(fib(0, 1, 16), fib(0, 1, 8))
    }
}
