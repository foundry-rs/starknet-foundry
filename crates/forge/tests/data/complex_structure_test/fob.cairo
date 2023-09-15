// Won't be found by the collector

use test_multiple::fob::fob_impl::fob_fn;

#[test]
fn test_fob() {
    assert(fob_fn(0, 1, 10) == 55, fob_fn(0, 1, 10));
}
