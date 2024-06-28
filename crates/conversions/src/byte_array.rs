use crate as conversions; // trick for CairoDeserialize macro
use crate::{serde::serialize::SerializeToFeltVec, string::TryFromHexStr};
use cairo_lang_runner::short_string::as_cairo_short_string_ex;
use cairo_lang_utils::byte_array::{BYTES_IN_WORD, BYTE_ARRAY_MAGIC};
use cairo_serde_macros::{CairoDeserialize, CairoSerialize};
use starknet_types_core::felt::Felt as Felt252;

#[derive(CairoDeserialize, CairoSerialize, Clone, Debug, PartialEq)]
pub struct ByteArray {
    words: Vec<Felt252>,
    pending_word: Felt252,
    pending_word_len: usize,
}

impl From<&str> for ByteArray {
    fn from(value: &str) -> Self {
        let chunks = value.as_bytes().chunks_exact(BYTES_IN_WORD);
        let remainder = chunks.remainder();
        let pending_word_len = remainder.len();

        let words = chunks.map(Felt252::from_bytes_be_slice).collect();
        let pending_word = Felt252::from_bytes_be_slice(remainder);

        Self {
            words,
            pending_word,
            pending_word_len,
        }
    }
}

impl ByteArray {
    #[must_use]
    pub fn serialize_with_magic(&self) -> Vec<Felt252> {
        let mut result = self.serialize_to_vec();

        result.insert(
            0,
            Felt252::try_from_hex_str(&format!("0x{BYTE_ARRAY_MAGIC}")).unwrap(),
        );

        result
    }
}

impl From<ByteArray> for String {
    fn from(value: ByteArray) -> Self {
        let full_words_string = value
            .words
            .iter()
            .map(|word| as_cairo_short_string_ex(word, BYTES_IN_WORD).unwrap())
            .collect::<String>();

        let pending_word_string =
            as_cairo_short_string_ex(&value.pending_word, value.pending_word_len).unwrap();

        format!("{full_words_string}{pending_word_string}")
    }
}
