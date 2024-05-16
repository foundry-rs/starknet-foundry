use crate::{felt252::SerializeAsFelt252Vec, string::TryFromHexStr};
use cairo_felt::Felt252;
use cairo_lang_utils::byte_array::{BYTES_IN_WORD, BYTE_ARRAY_MAGIC};

#[derive(Clone)]
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

        let words = chunks.map(Felt252::from_bytes_be).collect();
        let pending_word = Felt252::from_bytes_be(remainder);

        Self {
            words,
            pending_word,
            pending_word_len,
        }
    }
}

impl SerializeAsFelt252Vec for ByteArray {
    fn serialize_into_felt252_vec(&self, output: &mut Vec<Felt252>) {
        self.words.serialize_into_felt252_vec(output);
        self.pending_word.serialize_into_felt252_vec(output);

        output.push(self.pending_word_len.into());
    }
}

impl ByteArray {
    #[must_use]
    pub fn serialize_with_magic(&self) -> Vec<Felt252> {
        let mut result = self.serialize_no_magic();

        result.insert(
            0,
            Felt252::try_from_hex_str(&format!("0x{BYTE_ARRAY_MAGIC}")).unwrap(),
        );

        result
    }

    #[must_use]
    pub fn serialize_no_magic(&self) -> Vec<Felt252> {
        self.serialize_as_felt252_vec()
    }
}
