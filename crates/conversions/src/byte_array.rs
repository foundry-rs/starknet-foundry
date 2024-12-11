use crate as conversions; // trick for CairoDeserialize macro
use crate::serde::deserialize::{BufferReadError, BufferReadResult, BufferReader};
use crate::{serde::serialize::SerializeToFeltVec, string::TryFromHexStr};
use cairo_lang_runner::short_string::as_cairo_short_string_ex;
use cairo_lang_utils::byte_array::{BYTES_IN_WORD, BYTE_ARRAY_MAGIC};
use cairo_serde_macros::{CairoDeserialize, CairoSerialize};
use serde::{Deserialize, Serialize};
use starknet_types_core::felt::Felt;
use std::fmt;

#[derive(Serialize, Deserialize, CairoDeserialize, CairoSerialize, Clone, Debug, PartialEq)]
pub struct ByteArray {
    words: Vec<Felt>,
    pending_word: Felt,
    pending_word_len: usize,
}

impl From<&str> for ByteArray {
    fn from(value: &str) -> Self {
        let chunks = value.as_bytes().chunks_exact(BYTES_IN_WORD);
        let remainder = chunks.remainder();
        let pending_word_len = remainder.len();

        let words = chunks.map(Felt::from_bytes_be_slice).collect();
        let pending_word = Felt::from_bytes_be_slice(remainder);

        Self {
            words,
            pending_word,
            pending_word_len,
        }
    }
}

impl ByteArray {
    #[must_use]
    pub fn new(words: Vec<Felt>, pending_word: Felt, pending_word_len: usize) -> Self {
        Self {
            words,
            pending_word,
            pending_word_len,
        }
    }
}

impl ByteArray {
    #[must_use]
    pub fn serialize_with_magic(&self) -> Vec<Felt> {
        let mut result = self.serialize_to_vec();

        result.insert(
            0,
            Felt::try_from_hex_str(&format!("0x{BYTE_ARRAY_MAGIC}")).unwrap(),
        );

        result
    }

    pub fn deserialize_with_magic(value: &[Felt]) -> BufferReadResult<ByteArray> {
        if value.first() == Some(&Felt::try_from_hex_str(&format!("0x{BYTE_ARRAY_MAGIC}")).unwrap())
        {
            BufferReader::new(&value[1..]).read()
        } else {
            Err(BufferReadError::ParseFailed)
        }
    }
}

impl fmt::Display for ByteArray {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let full_words_string = self
            .words
            .iter()
            .map(|word| as_cairo_short_string_ex(word, BYTES_IN_WORD).unwrap())
            .collect::<String>();

        let pending_word_string =
            as_cairo_short_string_ex(&self.pending_word, self.pending_word_len).unwrap();

        write!(f, "{full_words_string}{pending_word_string}")
    }
}
