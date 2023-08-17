use addition::add;

fn fib(a: felt252, b: felt252, n: felt252) -> felt252 {
    match n {
        0 => a,
        _ => fib(b, add(a, b), n - 1),
    }
}

#[cfg(test)]
mod tests {
    use super::fib;

    #[test]
    #[available_gas(100000)]
    fn it_works() {
        assert(fib(0, 1, 16) == 987, 'it works!');
    }
}
