/// Simple test file for testing --test-files flag functionality
/// This file contains basic test helper functions

/// Test function that validates a felt252 value
fn test_validate_felt(value: felt252) -> bool {
    value != 0
}

/// Test function that adds two felt252 values
fn test_add_felts(a: felt252, b: felt252) -> felt252 {
    a + b
}

/// Test function that compares two felt252 values
fn test_compare_felts(a: felt252, b: felt252) -> bool {
    a == b
}