use test_case::add;

#[test]
#[test_case(1, 2, 3)]
#[test_case(3, 4, 7)]
#[test_case(5, 6, 11)]
fn simple_addition(a: felt252, b: felt252, expected: felt252) {
    let result = add(a, b);
    assert!(result == expected);
}

#[test]
#[test_case(name: "one_and_two", 1, 2, 3)]
#[test_case(name: "three_and_four", 3, 4, 7)]
#[test_case(name: "five_and_six", 5, 6, 11)]
fn addition_with_name_arg(a: felt252, b: felt252, expected: felt252) {
    let result = add(a, b);
    assert!(result == expected);
}

