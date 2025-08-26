use test_case::add;

#[test]
#[test_case(1, 2)]
#[test_case(3, 4)]
#[available_gas(l2_gas: 40000000)]
fn with_available_gas(a: felt252, b: felt252) {
    add(a, b);
}

#[test]
#[test_case(1, 2)]
#[test_case(3, 4)]
#[available_gas(l2_gas: 10000)]
fn with_available_gas_exceed_limit(a: felt252, b: felt252) {
    add(a, b);
}


#[test]
#[test_case(1, 2)]
#[test_case(3, 4)]
#[should_panic(expected: 'panic message')]
fn with_should_panic(a: felt252, b: felt252) {
    let x: i8 = -1;
    assert(x > 0, 'panic message');
}

#[test]
#[test_case(1, 2)]
#[test_case(3, 4)]
#[fuzzer]
fn with_fuzzer(a: felt252, b: felt252) {
    add(a, b);
}


#[test]
#[test_case(1, 2)]
#[test_case(3, 4)]
#[ignore]
fn with_ignore(a: felt252, b: felt252) {
    add(a, b);
}
