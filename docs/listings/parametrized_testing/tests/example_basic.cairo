fn sum(x: felt252, y: felt252) -> felt252 {
    return x + y;
}

#[test]
#[test_case(1, 2, 3)]
#[test_case(3, 4, 7)]
fn test_basic_sum(x: felt252, y: felt252, expected: felt252) {
    assert_eq!(sum(x, y), expected);
}
