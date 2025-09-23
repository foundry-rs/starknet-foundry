fn sum(x: felt252, y: felt252) -> felt252 {
    return x + y;
}

#[test]
#[test_case(1, 2)]
#[test_case(3, 4)]
#[fuzzer(runs: 10)]
fn test_sum(x: felt252, y: felt252) {
    assert_eq!(sum(x, y), x + y);
}
