use test_multiple::fib::fib_fn;

fn fab_fn(a: felt252, b: felt252, n: felt252) -> felt252 {
    match n {
        0 => a,
        _ => fab_fn(b, a + b, n - 1),
    }
}

#[test]
fn test_fab() {
    assert(fab_fn(0, 1, 10) == 55, fab_fn(0, 1, 10));
}

#[test]
fn test_how_does_this_work() {
    assert(fib_fn(0, 1, 10) == 55, fib_fn(0, 1, 10));
}
