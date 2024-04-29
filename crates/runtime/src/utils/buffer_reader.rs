use super::from_reader::FromReader;
use crate::EnhancedHintError;
use cairo_felt::Felt252;
use cairo_lang_runner::short_string::as_cairo_short_string;
use indoc::indoc;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum BufferReadError {
    #[error("Read out of bounds")]
    EndOfBuffer,
    #[error("Failed to parse while reading")]
    ParseFailed,
}

impl From<BufferReadError> for EnhancedHintError {
    fn from(value: BufferReadError) -> Self {
        EnhancedHintError::Anyhow(
            anyhow::Error::from(value)
                .context(
                    indoc!(r"
                        Reading from buffer failed, this can be caused by calling starknet::testing::cheatcode with invalid arguments.
                        Probably `snforge_std`/`sncast_std` version is incompatible, check above for incompatibility warning.
                    ")
                )
        )
    }
}

pub struct BufferReader<'a> {
    pub buffer: &'a [Felt252],
    pub idx: usize,
}

pub type BufferReadResult<T> = core::result::Result<T, BufferReadError>;

impl<'b> BufferReader<'b> {
    #[must_use]
    pub fn new<'a>(buffer: &'a [Felt252]) -> BufferReader<'a> {
        BufferReader::<'a> { buffer, idx: 0 }
    }

    pub fn read<T>(&mut self) -> BufferReadResult<T>
    where
        T: FromReader,
    {
        T::from_reader(self)
    }

    pub fn read_short_string(&mut self) -> BufferReadResult<Option<String>> {
        self.read::<Felt252>()
            .map(|felt| as_cairo_short_string(&felt))
    }
}
