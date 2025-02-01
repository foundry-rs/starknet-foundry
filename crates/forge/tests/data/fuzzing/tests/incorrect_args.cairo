use fuzzing::adder;

#[derive(Drop)]
struct MyStruct {
    a: felt252
}

#[test]
fn correct_args(b: felt252) {
    let result = adder(2, b);
    assert(result == 2 + b, '2 + b == 2 + b');
}

// #[test]
// fn incorrect_args(b: felt252, a: MyStruct) {
//     let result = adder(2, b);
//     assert(result == 2 + b, '2 + b == 2 + b');
// }
