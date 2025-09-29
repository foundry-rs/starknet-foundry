use subtraction::subtract;

fn fib(a: felt252, b: felt252, n: felt252) -> felt252 {
    match n {
        0 => a,
        _ => fib(b, subtract(a, -b), n - 1),
    }
}

#[starknet::interface]
trait IFibonacciContract<TContractState> {
    fn answer(ref self: TContractState) -> felt252;
}

#[starknet::contract]
mod FibonacciContract {
    use subtraction::subtract;
    use super::fib;

    #[storage]
    struct Storage {}

    #[abi(embed_v0)]
    impl FibonacciContractImpl of super::IFibonacciContract<ContractState> {
        fn answer(ref self: ContractState) -> felt252 {
            subtract(fib(0, 1, 16), fib(0, 1, 8))
        }
    }
}

#[cfg(test)]
mod tests {
    use snforge_std::declare;
    use super::fib;

    #[test]
    fn it_works() {
        assert(fib(0, 1, 16) == 987, 'it works!');
    }

    #[test]
    fn contract_test() {
        declare("FibonacciContract").unwrap();
        declare("SubtractionContract").unwrap();
    }
}
