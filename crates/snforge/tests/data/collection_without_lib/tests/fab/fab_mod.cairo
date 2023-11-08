use super::fab_fn;

#[test]
fn test_fab() {
    assert(fab_fn(0, 1, 10) == 55, fab_fn(0, 1, 10));
}
