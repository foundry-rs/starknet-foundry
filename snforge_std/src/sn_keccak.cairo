use keccak::keccak_u256s_be_inputs;
use traits::{Into, TryInto};
use array::ArrayTrait;
use option::OptionTrait;
use super::PrintTrait;

fn shortstring_keccak(shortstring: felt252) -> felt252 {
    let input: u256 = shortstring.into();

    let res = keccak_u256s_be_inputs(array![input].span());

    let reversed_low = integer::u128_byte_reverse(res.low);
    let reversed_high = integer::u128_byte_reverse(res.high);
    let pow_2_122: u128 = 0x4000000000000000000000000000000;

    let u256_be = u256 {
        low: reversed_high,
        high: reversed_low - (reversed_low / pow_2_122) * pow_2_122
    };

    u256_be.try_into().unwrap()
}
