use test_case::add;

#[test]
#[test_case(1, 2)]
#[test_case(3, 4)]
#[test_case(5, 6)]
fn simple_addition(a: felt252, b: felt252) {
    add(a, b);
}

