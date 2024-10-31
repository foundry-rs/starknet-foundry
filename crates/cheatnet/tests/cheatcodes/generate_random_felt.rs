use cairo_vm::Felt252;
use num_bigint::BigUint;
use num_traits::One;

use cheatnet::runtime_extensions::forge_runtime_extension::cheatcodes::generate_random_felt::generate_random_felt;

#[test]
fn test_random_felt252_range() {
    // Check that the number is less than 2^252
    let max_felt252: Felt252 = Felt252::from(BigUint::one() << 252);
    assert!(
        generate_random_felt() < max_felt252,
        "Number is out of felt252 range"
    );
    assert!(
        generate_random_felt() < max_felt252,
        "Number is out of felt252 range"
    );
    assert!(
        generate_random_felt() < max_felt252,
        "Number is out of felt252 range"
    );
    assert!(
        generate_random_felt() < max_felt252,
        "Number is out of felt252 range"
    );
    assert!(
        generate_random_felt() < max_felt252,
        "Number is out of felt252 range"
    );
}

#[test]
fn test_random_felt252_uniqueness() {
    let random_number1: Felt252 = generate_random_felt();
    let random_number2: Felt252 = generate_random_felt();

    // Check that two consecutive calls don't produce the same number
    assert!(
        random_number1 != random_number2,
        "Random numbers should not be identical"
    );
}
