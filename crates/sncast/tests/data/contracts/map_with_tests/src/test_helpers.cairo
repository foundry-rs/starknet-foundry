/// Test helper functions for the map contract
/// These functions can be used by test files to validate contract behavior

/// Helper function that creates a test key-value pair
fn create_test_pair(key: felt252, value: felt252) -> (felt252, felt252) {
    (key, value)
}

/// Helper function that validates a key is not zero
fn validate_key(key: felt252) -> bool {
    key != 0
}

/// Helper function that validates a value is positive
fn validate_value(value: felt252) -> bool {
    value > 0
}

/// Helper function that generates a test key from an index
fn generate_test_key(index: felt252) -> felt252 {
    index * 100
}

/// Helper function that generates a test value from an index
fn generate_test_value(index: felt252) -> felt252 {
    index * 10
}