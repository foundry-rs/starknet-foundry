use starknet::testing::cheatcode;
use super::super::byte_array::byte_array_as_felt_array;

/// Reads an environment variable, without parsing it
/// `name` - name of an environment variable
/// Returns the read span of felts
fn var(name: ByteArray) -> Span<felt252> {
    cheatcode::<'var'>(byte_array_as_felt_array(@name).span())
}
