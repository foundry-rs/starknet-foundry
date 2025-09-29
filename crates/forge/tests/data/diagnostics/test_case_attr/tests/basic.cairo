#[test]
#[test_case(3, 4, 7)]
fn function_without_params() {
    let _x = 10;
}

#[test]
#[test_case(3, 4, 7)]
fn function_with_invalid_params_count(x: felt252, y: felt252) {
    let _x = 10;
}

#[test]
#[test_case(name: array![1, 2, 3], 3, 4, 7)]
fn invalid_name_arg(x: felt252, y: felt252, expected: felt252) {
    let _x = 10;
}
