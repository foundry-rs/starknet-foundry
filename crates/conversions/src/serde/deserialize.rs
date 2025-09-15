use starknet_types_core::felt::Felt;
use thiserror::Error;

pub use cairo_serde_macros::CairoDeserialize;

mod deserialize_impl;

#[derive(Error, Debug)]
pub enum BufferReadError {
    #[error("Read out of bounds")]
    EndOfBuffer,
    #[error("Failed to parse while reading")]
    ParseFailed,
}

pub type BufferReadResult<T> = Result<T, BufferReadError>;

pub struct BufferReader<'a> {
    buffer: &'a [Felt],
}

pub trait CairoDeserialize: Sized {
    fn deserialize(reader: &mut BufferReader<'_>) -> BufferReadResult<Self>;
}

impl<'a> BufferReader<'a> {
    #[must_use]
    pub fn new(buffer: &'a [Felt]) -> Self {
        Self { buffer }
    }

    pub fn read_felt(&mut self) -> BufferReadResult<Felt> {
        let [head, tail @ ..] = self.buffer else {
            return Err(BufferReadError::EndOfBuffer);
        };
        self.buffer = tail;
        Ok(*head)
    }

    #[must_use]
    pub fn into_remaining(self) -> &'a [Felt] {
        self.buffer
    }

    pub fn read<T>(&mut self) -> BufferReadResult<T>
    where
        T: CairoDeserialize,
    {
        T::deserialize(self)
    }
}
