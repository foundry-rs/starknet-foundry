fn fib(a: felt252, b: felt252, n: felt252) -> felt252 {
    match n {
        0 => a,
        _ => fib(b, a + b, n - 1),
    }
}

#[test]
fn test_fib() {
    assert(fib(0, 1, 10) == 55, fib(0, 1, 10));
}
