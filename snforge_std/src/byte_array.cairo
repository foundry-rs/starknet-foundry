use core::byte_array::BYTE_ARRAY_MAGIC;

fn byte_array_as_felt_array(self: @ByteArray) -> Array<felt252> {
    let mut serialized = array![core::byte_array::BYTE_ARRAY_MAGIC];

    self.serialize(ref serialized);

    serialized
}

fn try_deserialize_bytearray_error(x: Array<felt252>) -> Result<ByteArray, ()> {
    if x.len() > 0 && *x.at(0) == BYTE_ARRAY_MAGIC {
        let mut x_span = x.span().slice(1, x.len() - 1);
        let deserialized = match Serde::<ByteArray>::deserialize(ref x_span) {
            Option::Some(str) => str,
            Option::None => panic!("panic string not deserializable")
        };
        return Result::Ok(deserialized);
    }
    Result::Err(())
}