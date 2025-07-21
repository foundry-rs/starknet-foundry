/// Simple test file for testing --test-files flag functionality
/// This file contains basic test helper functions

/// Test function that validates a felt252 value
#[test]
fn test_validate_felt(value: felt252) -> bool {
    value != 0
}
