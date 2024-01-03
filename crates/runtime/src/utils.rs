use cairo_felt::Felt252;
use cairo_lang_runner::short_string::as_cairo_short_string;
use num_traits::cast::ToPrimitive;
use num_traits::identities::One;

pub struct Reader<'a> {
    pub buffer: &'a [Felt252],
    pub idx: &'a mut usize,
}

impl Reader<'_> {
    pub fn new<'a>(buffer: &'a [Felt252], idx: &'a mut usize) -> Reader<'a> {
        Reader::<'a> { buffer, idx }
    }

    pub fn read_felt(&mut self) -> Felt252 {
        *self.idx += 1;
        self.buffer[*self.idx - 1].clone()
    }

    pub fn read_vec(&mut self, count: usize) -> Vec<Felt252> {
        *self.idx += count;
        self.buffer[*self.idx - count..*self.idx].to_vec()
    }

    pub fn read_option_felt(&mut self) -> Option<Felt252> {
        *self.idx += 1;
        (!self.buffer[*self.idx - 1].is_one()).then(|| self.read_felt())
    }

    pub fn read_option_vec(&mut self) -> Option<Vec<Felt252>> {
        self.read_option_felt()
            .map(|count| self.read_vec(count.to_usize().expect("Invalid Vec length value")))
    }

    pub fn read_bool(&mut self) -> bool {
        *self.idx += 1;
        self.buffer[*self.idx - 1] == 1.into()
    }

    pub fn read_short_string(&mut self) -> Option<String> {
        as_cairo_short_string(&self.read_felt())
    }
}
