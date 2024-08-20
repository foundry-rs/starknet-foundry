use starknet_types_core::felt::Felt as Felt252;

pub use cairo_serde_macros::CairoSerialize;

pub mod raw;
mod serialize_impl;

pub struct BufferWriter {
    output: Vec<Felt252>,
}

impl BufferWriter {
    fn new() -> Self {
        Self { output: vec![] }
    }

    pub fn write_felt(&mut self, felt: Felt252) {
        self.output.push(felt);
    }

    pub fn write<T>(&mut self, serializable: T)
    where
        T: CairoSerialize,
    {
        serializable.serialize(self);
    }

    #[must_use]
    pub fn to_vec(self) -> Vec<Felt252> {
        self.output
    }
}

pub trait CairoSerialize {
    fn serialize(&self, output: &mut BufferWriter);
}

pub trait SerializeToFeltVec {
    fn serialize_to_vec(&self) -> Vec<Felt252>;
}

impl<T> SerializeToFeltVec for T
where
    T: CairoSerialize,
{
    fn serialize_to_vec(&self) -> Vec<Felt252> {
        let mut buffer = BufferWriter::new();

        self.serialize(&mut buffer);

        buffer.to_vec()
    }
}
