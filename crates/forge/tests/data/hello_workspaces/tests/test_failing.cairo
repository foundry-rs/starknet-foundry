#[test]
fn test_failing() {
    assert(1 == 2, 'failing check');
}

#[test]
fn test_another_failing() {
    assert(2 == 3, 'failing check');
}
