mod fib;
mod fob;
mod fab;

use fob::fob_impl::fob_fn;
use fib::fib_fn;

#[test]
fn test_simple() {
    assert(1 == 1, 1);
}

#[test]
fn test_fob_in_lib() {
    assert(fob_fn(0, 1, 10) == 55, fob_fn(0, 1, 10));
}

#[test]
fn test_fib_in_lib() {
    assert(fib_fn(0, 1, 10) == 55, fib_fn(0, 1, 10));
}
