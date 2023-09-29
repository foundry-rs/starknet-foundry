use fuzzing::adder;

#[test]
fn correct_args(b: felt252) {
    let result = adder(2, b);
    assert(result == 2 + b, '2 + b == 2 + b');
}

#[test]
fn incorrect_args(b: felt252, a: u8) {
    let result = adder(2, b);
    assert(result == 2 + b, '2 + b == 2 + b');
}
