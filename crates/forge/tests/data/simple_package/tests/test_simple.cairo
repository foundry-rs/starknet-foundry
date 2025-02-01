#[test]
fn test_simple() {
    assert(1 == 1, 'simple check');
}

#[test]
fn test_simple2() {
    assert(3 == 3, 'simple check');
}

#[test]
fn test_two() {
    assert(2 == 2, '2 == 2');
}

#[test]
fn test_two_and_two() {
    assert(2 == 2, '2 == 2');
    assert(2 == 2, '2 == 2');
}

#[test]
#[should_panic(expected: 'failing check')]
fn test_failing() {
    assert(1 == 2, 'failing check');
}

#[test]
#[should_panic(expected: 'failing check')]
fn test_another_failing() {
    assert(2 == 3, 'failing check');
}
