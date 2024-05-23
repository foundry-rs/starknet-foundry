fn byte_array_as_felt_array(self: @ByteArray) -> Array<felt252> {
    let mut serialized = array![];

    self.serialize(ref serialized);

    serialized
}
