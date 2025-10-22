use conversions::serde::deserialize::{BufferReadResult, BufferReader, CairoDeserialize};
use conversions::serde::serialize::{BufferWriter, CairoSerialize};
use starknet_types_core::felt::Felt;

/// use this wrapper to NOT add extra length felt
/// useful e.g. when you need to pass an already serialized value
#[derive(Debug)]
pub struct NoLengthFeltVec<T>(pub Vec<T>)
where
    T: CairoSerialize;

impl<T> NoLengthFeltVec<T>
where
    T: CairoSerialize,
{
    #[must_use]
    pub fn new(vec: Vec<T>) -> Self {
        Self(vec)
    }
}

impl<T> CairoSerialize for NoLengthFeltVec<T>
where
    T: CairoSerialize,
{
    fn serialize(&self, output: &mut BufferWriter) {
        for e in &self.0 {
            e.serialize(output);
        }
    }
}

impl CairoDeserialize for NoLengthFeltVec<Felt> {
    fn deserialize(reader: &mut BufferReader<'_>) -> BufferReadResult<Self> {
        let mut result: Vec<Felt> = Vec::new();
        while let Ok(r) = reader.read_felt() {
            result.push(r);
        }
        Ok(Self::new(result))
    }
}
