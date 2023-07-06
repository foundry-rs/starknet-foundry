#[test]
fn test_simple() {
    assert(1 == 1, 'simple check');
}

#[test]
fn test_simple2() {
    assert(3 == 3, 'simple check');
}

#[test]
fn test_early_failing() {
    assert(1 == 2, 'failing check');
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
fn test_failing() {
    assert(1 == 2, 'failing check');
}
