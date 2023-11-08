// Won't be found by the collector

use collection_with_lib::fib::fib_fn;

#[test]
fn test_fib() {
    assert(fib_fn(0, 1, 10) == 55, fib_fn(0, 1, 10));
}
