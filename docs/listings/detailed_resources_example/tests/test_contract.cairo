#[test]
fn test_abc() {
    assert(1 == 1, 1);
}

#[test]
#[should_panic(expected: 'failing check' )]
fn test_failing() {
    assert(1 == 2, 'failing check');
}

#[test]
fn test_xyz() {
    assert(1 == 1, 1);
}

