use cairo_felt::Felt252;
use cairo_lang_utils::byte_array::{BYTES_IN_WORD, BYTE_ARRAY_MAGIC};
use itertools::chain;
use num_traits::Num;

pub struct ByteArray {
    words: Vec<Felt252>,
    pending_word_len: usize,
    pending_word: Felt252,
}

impl From<String> for ByteArray {
    fn from(value: String) -> Self {
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

impl ByteArray {
    #[must_use]
    pub fn serialize_with_magic(self) -> Vec<Felt252> {
        chain!(
            [Felt252::from_str_radix(BYTE_ARRAY_MAGIC, 16).unwrap(),],
            self.serialize().into_iter()
        )
        .collect()
    }

    #[must_use]
    pub fn serialize(self) -> Vec<Felt252> {
        chain!(
            [self.words.len().into()],
            self.words.into_iter(),
            [self.pending_word, self.pending_word_len.into()]
        )
        .collect()
    }
}
