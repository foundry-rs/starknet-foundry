#[test]
fn efg_test() {
    assert(super::foo() == 1, '');
}

#[test]
#[should_panic(expected: '')]
fn failing_test() {
    assert(1 == 2, '');
}
