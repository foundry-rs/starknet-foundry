use crate as conversions; // trick for CairoDeserialize macro
use crate::serde::deserialize::{BufferReadError, BufferReadResult, BufferReader};
use crate::{serde::serialize::SerializeToFeltVec, string::TryFromHexStr};
use cairo_lang_utils::byte_array::{BYTE_ARRAY_MAGIC, BYTES_IN_WORD};
use cairo_serde_macros::{CairoDeserialize, CairoSerialize};
use starknet_types_core::felt::Felt;
use std::fmt;

#[derive(CairoDeserialize, CairoSerialize, Clone, Debug, PartialEq)]
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

fn get_pending_word_bytes(word: &Felt, len: usize) -> Vec<u8> {
    word.to_bytes_be()[(32 - len)..32].to_vec()
}

fn get_full_word_bytes(word: &Felt) -> Vec<u8> {
    word.to_bytes_be()[1..32].to_vec()
}

impl fmt::Display for ByteArray {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut bytes: Vec<u8> = self
            .words
            .iter()
            .flat_map(get_full_word_bytes)
            .collect::<Vec<u8>>();
        bytes.extend(get_pending_word_bytes(
            &self.pending_word,
            self.pending_word_len,
        ));
        let string = String::from_utf8_lossy(&bytes).to_string();
        write!(f, "{string}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fmt_empty() {
        let array = ByteArray::from("");
        assert_eq!(array.to_string(), "");
    }

    #[test]
    fn test_fmt_single_word() {
        let array = ByteArray::from("Hello");
        assert_eq!(array.to_string(), "Hello");
    }

    #[test]
    fn test_fmt_multiple_words() {
        let array = ByteArray::from("Hello World! This is a test.");
        assert_eq!(array.to_string(), "Hello World! This is a test.");
    }

    #[test]
    fn test_fmt_with_pending_word() {
        let array = ByteArray::from("abc");
        assert_eq!(array.to_string(), "abc");
    }

    #[test]
    fn test_fmt_special_chars() {
        let special_chars = "!@#$%^&*()_+-=[]{}|;:,.<>?";
        let array = ByteArray::from(special_chars);
        assert_eq!(array.to_string(), special_chars);
    }

    #[test]
    fn test_fmt_with_null_bytes() {
        let with_nulls = "Hello\0World\0Test";
        let array = ByteArray::from(with_nulls);
        assert_eq!(array.to_string(), with_nulls);
    }

    #[test]
    fn test_fmt_mixed_ascii() {
        let mixed = "Hello\tWorld\n123 !@#";
        let array = ByteArray::from(mixed);
        assert_eq!(array.to_string(), mixed);
    }

    #[test]
    fn test_fmt_with_newlines() {
        let with_newlines = "First line\nSecond line\r\nThird line";
        let array = ByteArray::from(with_newlines);
        assert_eq!(array.to_string(), with_newlines);
    }

    #[test]
    fn test_fmt_multiple_newlines() {
        let multiple_newlines = "Line1\n\n\nLine2\n\nLine3";
        let array = ByteArray::from(multiple_newlines);
        assert_eq!(array.to_string(), multiple_newlines);
    }
}
