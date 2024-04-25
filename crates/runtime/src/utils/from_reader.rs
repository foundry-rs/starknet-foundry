use super::buffer_reader::{BufferReadError, BufferReadResult, BufferReader};
use cairo_felt::{felt_str, Felt252};
use cairo_lang_runner::short_string::as_cairo_short_string_ex;
use cairo_lang_utils::byte_array::{BYTES_IN_WORD, BYTE_ARRAY_MAGIC};
use conversions::IntoConv;
use num_traits::cast::ToPrimitive;
use num_traits::One;
use starknet::core::types::FieldElement;
use starknet_api::{
    core::{ClassHash, ContractAddress, EntryPointSelector, Nonce},
    hash::StarkFelt,
};

pub trait FromReader: Sized {
    fn from_reader(reader: &mut BufferReader<'_>) -> BufferReadResult<Self>;
}

// not blanked T: FromConv<Felt252> because in different crate than FromConv so conflicting implementations
macro_rules! impl_from_reader_for_felt_type {
    ($type:ty) => {
        impl FromReader for $type {
            fn from_reader(reader: &mut BufferReader<'_>) -> BufferReadResult<Self> {
                Felt252::from_reader(reader).map(IntoConv::into_)
            }
        }
    };
}

impl_from_reader_for_felt_type!(FieldElement);
impl_from_reader_for_felt_type!(ClassHash);
impl_from_reader_for_felt_type!(StarkFelt);
impl_from_reader_for_felt_type!(ContractAddress);
impl_from_reader_for_felt_type!(Nonce);
impl_from_reader_for_felt_type!(EntryPointSelector);

impl FromReader for Felt252 {
    fn from_reader(reader: &mut BufferReader<'_>) -> BufferReadResult<Self> {
        let felt = reader.buffer.get(reader.idx).cloned();

        reader.idx += 1;

        felt.ok_or(BufferReadError::EndOfBuffer)
    }
}

impl FromReader for Vec<Felt252> {
    fn from_reader(reader: &mut BufferReader<'_>) -> BufferReadResult<Self> {
        let length = reader.read_felt()?;
        let length = felt252_to_vec_length(&length)?;
        Ok(reader.read_slice(length)?.to_owned())
    }
}

impl FromReader for Option<Vec<Felt252>> {
    fn from_reader(reader: &mut BufferReader<'_>) -> BufferReadResult<Self> {
        Ok(match reader.read_option_felt()? {
            Some(count) => {
                let count = felt252_to_vec_length(&count)?;
                let result = reader.read_slice(count)?;

                Some(result.to_owned())
            }
            None => None,
        })
    }
}

impl FromReader for Option<Felt252> {
    fn from_reader(reader: &mut BufferReader<'_>) -> BufferReadResult<Self> {
        match reader.read_felt() {
            Ok(felt) if !felt.is_one() => Ok(Some(reader.read_felt()?)),
            _ => Ok(None),
        }
    }
}

impl FromReader for bool {
    fn from_reader(reader: &mut BufferReader<'_>) -> BufferReadResult<Self> {
        reader.read_felt().map(|felt| felt == 1.into())
    }
}

impl FromReader for String {
    fn from_reader(reader: &mut BufferReader<'_>) -> BufferReadResult<Self> {
        let (result, idx_increment) = try_format_string(
            reader
                .buffer
                .get(reader.idx..)
                .ok_or(BufferReadError::EndOfBuffer)?,
        )
        .ok_or(BufferReadError::ParseFailed)?;

        reader.idx += idx_increment;

        Ok(result)
    }
}

fn felt252_to_vec_length(vec_len: &Felt252) -> BufferReadResult<usize> {
    vec_len.to_usize().ok_or(BufferReadError::ParseFailed)
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
