#[test]
fn efg_test() {
    assert(super::foo() == 1, '');
}

#[test]
fn failing_test() {
    assert(1 == 2, '');
}
