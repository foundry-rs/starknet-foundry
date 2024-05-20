use cairo_felt::Felt252;
use cairo_lang_runner::short_string::as_cairo_short_string;
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
    buffer: &'a [Felt252],
    idx: usize,
}

pub trait CairoDeserialize: Sized {
    fn deserialize(reader: &mut BufferReader<'_>) -> BufferReadResult<Self>;
}

impl<'b> BufferReader<'b> {
    #[must_use]
    pub fn new<'a>(buffer: &'a [Felt252]) -> BufferReader<'a> {
        BufferReader::<'a> { buffer, idx: 0 }
    }

    pub fn read_felt(&mut self) -> BufferReadResult<Felt252> {
        let felt = self.buffer.get(self.idx).cloned();

        self.idx += 1;

        felt.ok_or(BufferReadError::EndOfBuffer)
    }

    pub fn read<T>(&mut self) -> BufferReadResult<T>
    where
        T: CairoDeserialize,
    {
        T::deserialize(self)
    }

    pub fn read_short_string(&mut self) -> BufferReadResult<Option<String>> {
        self.read::<Felt252>()
            .map(|felt| as_cairo_short_string(&felt))
    }
}
