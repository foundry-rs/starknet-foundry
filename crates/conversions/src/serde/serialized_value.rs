use conversions::serde::deserialize::{BufferReadResult, BufferReader, CairoDeserialize};
use conversions::serde::serialize::{BufferWriter, CairoSerialize};
use starknet_types_core::felt::Felt;

/// Represents an already serialized Vec of values.
///
/// Use this to omit adding extra felt for the length of the vector during serialization.
#[derive(Debug)]
pub struct SerializedValue<T>(pub Vec<T>)
where
    T: CairoSerialize;

impl<T> SerializedValue<T>
where
    T: CairoSerialize,
{
    #[must_use]
    pub fn new(vec: Vec<T>) -> Self {
        Self(vec)
    }
}

impl<T> CairoSerialize for SerializedValue<T>
where
    T: CairoSerialize,
{
    fn serialize(&self, output: &mut BufferWriter) {
        for e in &self.0 {
            e.serialize(output);
        }
    }
}

impl CairoDeserialize for SerializedValue<Felt> {
    fn deserialize(reader: &mut BufferReader<'_>) -> BufferReadResult<Self> {
        let mut result: Vec<Felt> = Vec::new();
        while let Ok(r) = reader.read_felt() {
            result.push(r);
        }
        Ok(Self::new(result))
    }
}
