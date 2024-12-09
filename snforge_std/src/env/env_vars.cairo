use starknet::testing::cheatcode;
use super::super::byte_array::byte_array_as_felt_array;
use super::super::_cheatcode::handle_cheatcode;

/// Reads an environment variable, without parsing it
/// `name` - name of an environment variable
/// Returns the read array of felts
pub fn var(name: ByteArray) -> Array<felt252> {
    let mut output_array: Array<felt252> = array![];
    let result = handle_cheatcode(cheatcode::<'var'>(byte_array_as_felt_array(@name).span()));
    output_array.append_span(result);
    output_array
}
