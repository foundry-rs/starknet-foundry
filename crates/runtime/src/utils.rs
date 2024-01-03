use cairo_felt::Felt252;
use num_traits::cast::ToPrimitive;
use num_traits::identities::One;

pub fn read_felt(buffer: &[Felt252], idx: &mut usize) -> Felt252 {
    *idx += 1;
    buffer[*idx - 1].clone()
}

pub fn read_vec(buffer: &[Felt252], idx: &mut usize, count: usize) -> Vec<Felt252> {
    *idx += count;
    buffer[*idx - count..*idx].to_vec()
}

pub fn read_option_felt(buffer: &[Felt252], idx: &mut usize) -> Option<Felt252> {
    *idx += 1;
    (!buffer[*idx - 1].is_one()).then(|| read_felt(buffer, idx))
}

pub fn read_option_vec(buffer: &[Felt252], idx: &mut usize) -> Option<Vec<Felt252>> {
    read_option_felt(buffer, idx).map(|count| read_vec(buffer, idx, count.to_usize().unwrap()))
}
