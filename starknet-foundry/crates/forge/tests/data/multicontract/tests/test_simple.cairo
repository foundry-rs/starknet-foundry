#[test]
fn test_simple() {
    assert(1 == 1, 'simple check');
}

#[test]
fn test_simple2() {
    assert(3 == 3, 'simple check');
}

#[test]
fn test_failing() {
    assert(1 == 2, 'failing check');
}
