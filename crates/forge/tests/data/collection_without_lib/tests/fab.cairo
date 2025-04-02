use collection_without_lib::fab::fab_impl::fab_fn;

mod fab_mod;

#[test]
fn test_fab() {
    assert(fab_fn(0, 1, 10) == 55, fab_fn(0, 1, 10));
}
