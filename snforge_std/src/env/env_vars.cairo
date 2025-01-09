use super::super::byte_array::byte_array_as_felt_array;
use super::super::_cheatcode::typed_checked_cheatcode;

/// Reads an environment variable, without parsing it
/// `name` - name of an environment variable
/// Returns the read array of felts
pub fn var(name: ByteArray) -> Array<felt252> {
    typed_checked_cheatcode::<'var', Array<felt252>>(byte_array_as_felt_array(@name).span())
}
