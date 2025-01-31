use core::byte_array::BYTE_ARRAY_MAGIC;

pub fn byte_array_as_felt_array(self: @ByteArray) -> Array<felt252> {
    let mut serialized = array![];

    self.serialize(ref serialized);

    serialized
}

/// This function is meant to transform a serialized output from a contract call into a `ByteArray`.
/// `x` - Span of `felt252`s returned from a contract call (panic data)
/// Returns the parsed `ByteArray`, or an `Err` if the parsing failed.
pub fn try_deserialize_bytearray_error(x: Span<felt252>) -> Result<ByteArray, ByteArray> {
    if x.len() > 0 && *x.at(0) == BYTE_ARRAY_MAGIC {
        let mut x_span = x.slice(1, x.len() - 1);
        return match Serde::<ByteArray>::deserialize(ref x_span) {
            Option::Some(x) => Result::Ok(x),
            Option::None => Result::Err("Malformed input provided"),
        };
    }
    Result::Err("Input is not a ByteArray-formatted error")
}
