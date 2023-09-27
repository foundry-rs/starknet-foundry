use collection_without_lib::fob::fob_impl::fob_fn;
use super::fab::fab_impl::fab_fn;

fn fib_fn(a: felt252, b: felt252, n: felt252) -> felt252 {
    match n {
        0 => a,
        _ => fib_fn(b, a + b, n - 1),
    }
}

#[test]
fn test_fib() {
    assert(fib_fn(0, 1, 10) == 55, fib_fn(0, 1, 10));
}

#[test]
fn test_fob_in_fib() {
    assert(fob_fn(0, 1, 10) == 55, fob_fn(0, 1, 10));
}

#[test]
fn test_fab_in_fib() {
    assert(fab_fn(0, 1, 10) == 55, fab_fn(0, 1, 10));
}
