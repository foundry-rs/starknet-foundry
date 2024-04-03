use crate::felt252::SerializeAsFelt252Vec;
use cairo_felt::Felt252;
use cairo_lang_utils::byte_array::{BYTES_IN_WORD, BYTE_ARRAY_MAGIC};
use num_traits::Num;

#[derive(Clone)]
pub struct ByteArray {
    words: Vec<Felt252>,
    pending_word_len: usize,
    pending_word: Felt252,
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
            pending_word_len,
            pending_word,
        }
    }
}

impl SerializeAsFelt252Vec for ByteArray {
    fn serialize_as_felt252(self, output: &mut Vec<Felt252>) {
        output.extend(self.serialize_no_magic());
    }
}

impl ByteArray {
    #[must_use]
    pub fn serialize_with_magic(self) -> Vec<Felt252> {
        self.serialize(true)
    }

    #[must_use]
    pub fn serialize_no_magic(self) -> Vec<Felt252> {
        self.serialize(false)
    }

    #[must_use]
    fn serialize(self, magic: bool) -> Vec<Felt252> {
        let mut result = Vec::with_capacity(self.words.len() + 3 + usize::from(magic));

        if magic {
            result.push(Felt252::from_str_radix(BYTE_ARRAY_MAGIC, 16).unwrap());
        }

        result.push(self.words.len().into());
        result.extend(self.words);
        result.push(self.pending_word);
        result.push(self.pending_word_len.into());

        result
    }
}
