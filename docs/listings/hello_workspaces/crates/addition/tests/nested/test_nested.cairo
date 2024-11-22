use super::foo;

#[test]
fn test_two() {
    assert(foo() == 2, 'foo() == 2');
}

#[test]
fn test_two_and_two() {
    assert(2 == 2, '2 == 2');
    assert(2 == 2, '2 == 2');
}
