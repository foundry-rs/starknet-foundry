use crate as conversions; // trick for CairoDeserialize macro
use crate::serde::deserialize::{BufferReadError, BufferReadResult, BufferReader};
use crate::{serde::serialize::SerializeToFeltVec, string::TryFromHexStr};
use cairo_lang_runner::short_string::as_cairo_short_string_ex;
use cairo_lang_utils::byte_array::{BYTES_IN_WORD, BYTE_ARRAY_MAGIC};
use cairo_serde_macros::{CairoDeserialize, CairoSerialize};
use starknet_types_core::felt::Felt as Felt252;
use std::fmt;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

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

    pub fn deserialize_with_magic(value: &[Felt252]) -> BufferReadResult<ByteArray> {
        if value.first()
            == Some(&Felt252::try_from_hex_str(&format!("0x{BYTE_ARRAY_MAGIC}")).unwrap())
        {
            BufferReader::new(&value[1..]).read()
        } else {
            Err(BufferReadError::ParseFailed)
        }
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


impl fmt::Display for ByteArray {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}") 
    }
}

// we can implement that or dont use Serialize/Deserialize in CallResult/CallFailure in rpc.rs
// I am not sure which is better so I leave this for now
impl Serialize for ByteArray {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = self.to_string();
        serializer.serialize_str(&s)
    }
}

impl<'de> Deserialize<'de> for ByteArray {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(ByteArray::from(s.as_str()))
    }
}