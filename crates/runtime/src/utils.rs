use cairo_felt::{felt_str, Felt252};
use cairo_lang_runner::short_string::{as_cairo_short_string, as_cairo_short_string_ex};
use cairo_lang_utils::byte_array::{BYTES_IN_WORD, BYTE_ARRAY_MAGIC};
use num_traits::{cast::ToPrimitive, identities::One};

pub struct BufferReader<'a> {
    pub buffer: &'a [Felt252],
    pub idx: usize,
}

impl BufferReader<'_> {
    #[must_use]
    pub fn new<'a>(buffer: &'a [Felt252]) -> BufferReader<'a> {
        BufferReader::<'a> { buffer, idx: 0 }
    }

    pub fn read_felt(&mut self) -> Felt252 {
        self.idx += 1;
        self.buffer[self.idx - 1].clone()
    }

    pub fn read_vec_body(&mut self, count: usize) -> Vec<Felt252> {
        self.idx += count;
        self.buffer[self.idx - count..self.idx].to_vec()
    }

    pub fn read_vec(&mut self) -> Vec<Felt252> {
        let count = felt252_to_vec_length(&self.read_felt());
        self.read_vec_body(count)
    }

    pub fn read_option_felt(&mut self) -> Option<Felt252> {
        self.idx += 1;
        (!self.buffer[self.idx - 1].is_one()).then(|| self.read_felt())
    }

    pub fn read_option_vec(&mut self) -> Option<Vec<Felt252>> {
        self.read_option_felt()
            .map(|count| self.read_vec_body(felt252_to_vec_length(&count)))
    }

    pub fn read_bool(&mut self) -> bool {
        self.idx += 1;
        self.buffer[self.idx - 1] == 1.into()
    }

    pub fn read_short_string(&mut self) -> Option<String> {
        as_cairo_short_string(&self.read_felt())
    }

    pub fn read_option_string(&mut self) -> Option<String> {
        let (result, idx_increment) = try_format_string(&self.buffer[self.idx..])?;

        self.idx += idx_increment;

        Some(result)
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
        .map(|word| as_cairo_short_string_ex(&word, BYTES_IN_WORD))
        .collect::<Option<Vec<String>>>()?
        .join("");
    let pending_word = values.next()?;
    let pending_word_len = values.next()?.to_usize()?;

    let pending_word_string = as_cairo_short_string_ex(&pending_word, pending_word_len)?;

    Some((
        format!("{full_words_string}{pending_word_string}"),
        num_full_words + 4, //4 calls to .next()
    ))
}
