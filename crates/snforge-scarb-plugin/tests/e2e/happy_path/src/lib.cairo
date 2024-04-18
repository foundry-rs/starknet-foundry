
#[test]
fn test_only() {
    assert(true, '');
}

fn non_test() {
    assert(true, '');
}

#[test]
fn with_args(a: felt252, b: u8) {
    assert(true, '');
}
