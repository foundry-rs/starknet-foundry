use crate::byte_array::ByteArray;
use crate::string::{ParseFeltError, TryFromDecStr, TryFromHexStr};
use crate::{FromConv, IntoConv};
use blockifier::execution::execution_utils::stark_felt_to_felt;
use cairo_felt::Felt252;
use num_traits::{Bounded, Num};
use starknet::core::types::FieldElement;
use starknet_api::core::{EntryPointSelector, Nonce};
use starknet_api::{
    core::{ClassHash, ContractAddress},
    hash::StarkFelt,
};

impl FromConv<FieldElement> for Felt252 {
    fn from_(value: FieldElement) -> Felt252 {
        Felt252::from_bytes_be(&value.to_bytes_be())
    }
}

impl FromConv<StarkFelt> for Felt252 {
    fn from_(value: StarkFelt) -> Felt252 {
        stark_felt_to_felt(value)
    }
}

impl FromConv<ClassHash> for Felt252 {
    fn from_(value: ClassHash) -> Felt252 {
        Felt252::from_bytes_be(value.0.bytes())
    }
}

impl FromConv<ContractAddress> for Felt252 {
    fn from_(value: ContractAddress) -> Felt252 {
        stark_felt_to_felt(*value.0.key())
    }
}

impl FromConv<Nonce> for Felt252 {
    fn from_(value: Nonce) -> Felt252 {
        stark_felt_to_felt(value.0)
    }
}

impl FromConv<EntryPointSelector> for Felt252 {
    fn from_(value: EntryPointSelector) -> Felt252 {
        value.0.into_()
    }
}

impl TryFromDecStr for Felt252 {
    fn try_from_dec_str(value: &str) -> Result<Felt252, ParseFeltError> {
        from_string(value, 10)
    }
}

impl TryFromHexStr for Felt252 {
    fn try_from_hex_str(value: &str) -> Result<Felt252, ParseFeltError> {
        from_string(value.strip_prefix("0x").unwrap_or(value), 16)
    }
}

fn from_string(value: &str, radix: u32) -> Result<Felt252, ParseFeltError> {
    match Felt252::from_str_radix(value, radix) {
        Ok(felt) if felt <= Felt252::max_value() => Ok(felt),
        _ => Err(ParseFeltError),
    }
}

pub trait FromShortString<T>: Sized {
    fn from_short_string(short_string: &str) -> Result<T, ParseFeltError>;
}

impl FromShortString<Felt252> for Felt252 {
    fn from_short_string(short_string: &str) -> Result<Felt252, ParseFeltError> {
        if short_string.len() <= 31 && short_string.is_ascii() {
            Ok(Felt252::from_bytes_be(short_string.as_bytes()))
        } else {
            Err(ParseFeltError)
        }
    }
}

pub trait SerializeAsFelt252Vec {
    fn serialize_as_felt252_vec(&self) -> Vec<Felt252>;
}

impl<T: SerializeAsFelt252Vec, E: SerializeAsFelt252Vec> SerializeAsFelt252Vec for Result<T, E> {
    fn serialize_as_felt252_vec(&self) -> Vec<Felt252> {
        match self {
            Ok(val) => {
                let mut res = vec![Felt252::from(0)];
                res.extend(val.serialize_as_felt252_vec());
                res
            }
            Err(err) => {
                let mut res = vec![Felt252::from(1)];
                res.extend(err.serialize_as_felt252_vec());
                res
            }
        }
    }
}

impl SerializeAsFelt252Vec for &str {
    fn serialize_as_felt252_vec(&self) -> Vec<Felt252> {
        ByteArray::from(*self).serialize_no_magic()
    }
}
