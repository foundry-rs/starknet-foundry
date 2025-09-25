pub mod hello_starknet;

pub fn fib(a: felt252, b: felt252, n: felt252) -> felt252 {
    match n {
        0 => a,
        _ => fib(b, a + b, n - 1),
    }
}

#[cfg(test)]
mod tests {
    use super::fib;

    #[test]
    fn test_fib() {
        assert(fib(0, 1, 10) == 55, fib(0, 1, 10));
    }

    #[test]
    #[ignore]
    fn ignored_test() {
        assert(1 == 1, 'passing');
    }
}
