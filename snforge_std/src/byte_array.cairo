fn byte_array_as_felt_array(self: @ByteArray) -> Array<felt252> {
    let mut serialized = array![core::byte_array::BYTE_ARRAY_MAGIC];

    self.serialize(ref serialized);

    serialized
}
