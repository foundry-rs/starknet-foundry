use super::super::byte_array::byte_array_as_felt_array;
use super::super::cheatcode::execute_cheatcode;

/// Reads an environment variable, without parsing it
/// `name` - name of an environment variable
/// Returns the read array of felts
pub fn var(name: ByteArray) -> Array<felt252> {
    execute_cheatcode::<'var'>(byte_array_as_felt_array(@name).span()).into()
}
