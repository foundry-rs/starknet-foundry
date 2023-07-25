// Won't be found by the collector

use test_multiple::fob::fob_impl::fob_fn;
use test_multiple::fab::fab_impl::fab_fn;
use test_multiple::fib::fib_fn;

#[test]
fn test_fib() {
    assert(fib_fn(0, 1, 10) == 55, fib_fn(0, 1, 10));
}

#[test]
fn test_fob() {
    assert(fob_fn(0, 1, 10) == 55, fob_fn(0, 1, 10));
}

#[test]
fn test_fab() {
    assert(fab_fn(0, 1, 10) == 55, fab_fn(0, 1, 10));
}
