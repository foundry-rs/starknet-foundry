use crate::EnhancedHintError;
use cairo_felt::{felt_str, Felt252};
use cairo_lang_runner::short_string::{as_cairo_short_string, as_cairo_short_string_ex};
use cairo_lang_utils::byte_array::{BYTES_IN_WORD, BYTE_ARRAY_MAGIC};
use indoc::indoc;
use num_traits::{cast::ToPrimitive, identities::One};
use thiserror::Error;

pub struct BufferReader<'a> {
    pub buffer: &'a [Felt252],
    pub idx: usize,
}

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
                        Probably snforge_std version is incompatible, check above for incompatibility warning.
                    ")
                )
        )
    }
}

pub type BufferReadResult<T> = core::result::Result<T, BufferReadError>;

impl BufferReader<'_> {
    #[must_use]
    pub fn new<'a>(buffer: &'a [Felt252]) -> BufferReader<'a> {
        BufferReader::<'a> { buffer, idx: 0 }
    }

    pub fn read_felt(&mut self) -> BufferReadResult<Felt252> {
        let felt = self.buffer.get(self.idx).cloned();

        self.idx += 1;

        felt.ok_or(BufferReadError::EndOfBuffer)
    }

    pub fn read_vec_body(&mut self, count: usize) -> BufferReadResult<Vec<Felt252>> {
        let start = self.idx;

        self.idx += count;

        Ok(self
            .buffer
            .get(start..self.idx)
            .ok_or(BufferReadError::EndOfBuffer)?
            .to_vec())
    }

    pub fn read_vec(&mut self) -> BufferReadResult<Vec<Felt252>> {
        let count = felt252_to_vec_length(&self.read_felt()?);
        self.read_vec_body(count)
    }

    pub fn read_option_felt(&mut self) -> BufferReadResult<Option<Felt252>> {
        match self.read_felt() {
            Ok(felt) if !felt.is_one() => Ok(Some(self.read_felt()?)),
            _ => Ok(None),
        }
    }

    pub fn read_option_vec(&mut self) -> BufferReadResult<Option<Vec<Felt252>>> {
        Ok(match self.read_option_felt()? {
            Some(count) => Some(self.read_vec_body(felt252_to_vec_length(&count))?),
            None => None,
        })
    }

    pub fn read_bool(&mut self) -> BufferReadResult<bool> {
        self.read_felt().map(|felt| felt == 1.into())
    }

    pub fn read_short_string(&mut self) -> BufferReadResult<Option<String>> {
        self.read_felt().map(|felt| as_cairo_short_string(&felt))
    }

    pub fn read_string(&mut self) -> BufferReadResult<String> {
        let (result, idx_increment) = try_format_string(
            self.buffer
                .get(self.idx..)
                .ok_or(BufferReadError::EndOfBuffer)?,
        )
        .ok_or(BufferReadError::ParseFailed)?;

        self.idx += idx_increment;

        Ok(result)
    }
}

fn felt252_to_vec_length(vec_len: &Felt252) -> usize {
    vec_len.to_usize().expect("Invalid Vec length value")
}

fn try_format_string(values: &[Felt252]) -> Option<(String, usize)> {
    let mut values = values.iter();

    if values.next()? != &felt_str!(BYTE_ARRAY_MAGIC, 16) {
        return None;
    }

    let num_full_words = values.next()?.to_usize()?;
    let full_words_string = values
        .by_ref()
        .take(num_full_words)
        .map(|word| as_cairo_short_string_ex(word, BYTES_IN_WORD))
        .collect::<Option<Vec<String>>>()?
        .join("");
    let pending_word = values.next()?;
    let pending_word_len = values.next()?.to_usize()?;

    let pending_word_string = as_cairo_short_string_ex(pending_word, pending_word_len)?;

    Some((
        format!("{full_words_string}{pending_word_string}"),
        num_full_words + 4, //4 calls to .next()
    ))
}
