use fuzzing::adder;

#[derive(Debug, Drop)]
struct MyStruct {
    a: felt252
}

#[test]
#[fuzzer]
fn correct_args(b: felt252) {
    let result = adder(2, b);
    assert(result == 2 + b, '2 + b == 2 + b');
}

#[cfg(feature: 'unimplemented')]
#[test]
#[fuzzer]
fn incorrect_args(b: felt252, a: MyStruct) {
    let result = adder(2, b);
    assert(result == 2 + b, '2 + b == 2 + b');
}
