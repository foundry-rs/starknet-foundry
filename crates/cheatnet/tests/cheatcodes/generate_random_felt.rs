use starknet_types_core::felt::Felt;
use std::collections::HashSet;

use cheatnet::runtime_extensions::forge_runtime_extension::cheatcodes::generate_random_felt::generate_random_felt;

#[test]
fn test_generate_random_felt_range_and_uniqueness() {
    let mut random_values = vec![];

    let max_felt: Felt = Felt::MAX;

    for _ in 0..10 {
        let random_value = generate_random_felt();
        assert!(random_value < max_felt, "Value out of range");
        random_values.push(random_value);
    }

    let unique_values: HashSet<_> = random_values.iter().collect();
    assert!(
        unique_values.len() > 1,
        "Random values should not all be identical."
    );
}
