use cairo_vm::Felt252;
use num_bigint::BigUint;
use num_traits::One;

use cheatnet::runtime_extensions::forge_runtime_extension::cheatcodes::generate_random_felt::generate_random_felt;

#[test]
fn test_generate_random_felt_range() {
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
fn test_generate_random_felt_uniqueness() {
    // Check consecutive calls don't produce the same number
    assert!(
        generate_random_felt() != generate_random_felt(),
        "Random numbers should not be identical"
    );
    assert!(
        generate_random_felt() != generate_random_felt(),
        "Random numbers should not be identical"
    );
    assert!(
        generate_random_felt() != generate_random_felt(),
        "Random numbers should not be identical"
    );
    assert!(
        generate_random_felt() != generate_random_felt(),
        "Random numbers should not be identical"
    );
    assert!(
        generate_random_felt() != generate_random_felt(),
        "Random numbers should not be identical"
    );
}
