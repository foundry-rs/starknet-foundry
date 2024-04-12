use crate::{
    byte_array::ByteArray,
    string::{TryFromDecStr, TryFromHexStr},
    FromConv, IntoConv,
};
use blockifier::execution::execution_utils::stark_felt_to_felt;
use cairo_felt::{Felt252, ParseFeltError};
use num_traits::Num;
use starknet::core::types::FieldElement;
use starknet_api::{
    core::{ClassHash, ContractAddress, EntryPointSelector, Nonce},
    hash::StarkFelt,
};
use std::vec;

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
        value.0.into_()
    }
}

impl FromConv<ContractAddress> for Felt252 {
    fn from_(value: ContractAddress) -> Felt252 {
        (*value.0.key()).into_()
    }
}

impl FromConv<Nonce> for Felt252 {
    fn from_(value: Nonce) -> Felt252 {
        value.0.into_()
    }
}

impl FromConv<EntryPointSelector> for Felt252 {
    fn from_(value: EntryPointSelector) -> Felt252 {
        value.0.into_()
    }
}

impl<T> TryFromDecStr for T
where
    T: FromConv<Felt252>,
{
    fn try_from_dec_str(value: &str) -> Result<T, ParseFeltError> {
        from_string(value, 10)
    }
}

impl<T> TryFromHexStr for T
where
    T: FromConv<Felt252>,
{
    fn try_from_hex_str(value: &str) -> Result<T, ParseFeltError> {
        let value = value.strip_prefix("0x").ok_or(ParseFeltError)?;

        from_string(value, 16)
    }
}

fn from_string<T>(value: &str, radix: u32) -> Result<T, ParseFeltError>
where
    T: FromConv<Felt252>,
{
    match Felt252::from_str_radix(value, radix) {
        Ok(felt) => Ok(T::from_(felt)),
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

pub trait TryInferFormat: Sized {
    /// Parses value from `hex string`, `dec string`, `quotted cairo shortstring `and `quotted cairo string`
    fn infer_format_and_parse(value: &str) -> Result<Vec<Self>, ParseFeltError>;
}

fn resolve(value: &str) -> String {
    value[1..value.len() - 1].replace("\\n", "\n")
}

impl TryInferFormat for Felt252 {
    fn infer_format_and_parse(value: &str) -> Result<Vec<Self>, ParseFeltError> {
        if value.starts_with('\'') && value.ends_with('\'') {
            let value = resolve(value).replace("\\'", "'");

            Felt252::from_short_string(&value).map(|felt| vec![felt])
        } else if value.starts_with('"') && value.ends_with('"') {
            let value = resolve(value).replace("\\\"", "\"");

            Ok(ByteArray::from(value.as_str()).serialize_no_magic())
        } else {
            Felt252::try_from_hex_str(value)
                .or_else(|_| Felt252::try_from_dec_str(value))
                .map(|felt| vec![felt])
        }
    }
}

pub trait SerializeAsFelt252Vec: Sized {
    fn serialize_into_felt252_vec(self, output: &mut Vec<Felt252>);
    fn serialize_as_felt252_vec(self) -> Vec<Felt252> {
        let mut result = vec![];
        self.serialize_into_felt252_vec(&mut result);
        result
    }
}

impl SerializeAsFelt252Vec for Vec<Felt252> {
    fn serialize_into_felt252_vec(self, output: &mut Vec<Felt252>) {
        output.extend(self);
    }

    fn serialize_as_felt252_vec(self) -> Vec<Felt252> {
        self
    }
}

impl<T: SerializeAsFelt252Vec, E: SerializeAsFelt252Vec> SerializeAsFelt252Vec for Result<T, E> {
    fn serialize_into_felt252_vec(self, output: &mut Vec<Felt252>) {
        match self {
            Ok(val) => {
                output.push(Felt252::from(0));
                val.serialize_into_felt252_vec(output);
            }
            Err(err) => {
                output.push(Felt252::from(1));
                err.serialize_into_felt252_vec(output);
            }
        }
    }
}

impl<T> SerializeAsFelt252Vec for T
where
    T: IntoConv<Felt252>,
{
    fn serialize_into_felt252_vec(self, output: &mut Vec<Felt252>) {
        output.push(self.into_());
    }

    fn serialize_as_felt252_vec(self) -> Vec<Felt252> {
        vec![self.into_()]
    }
}

impl SerializeAsFelt252Vec for &str {
    fn serialize_into_felt252_vec(self, output: &mut Vec<Felt252>) {
        output.extend(self.serialize_as_felt252_vec());
    }

    fn serialize_as_felt252_vec(self) -> Vec<Felt252> {
        ByteArray::from(self).serialize_no_magic()
    }
}

impl SerializeAsFelt252Vec for String {
    fn serialize_into_felt252_vec(self, output: &mut Vec<Felt252>) {
        self.as_str().serialize_into_felt252_vec(output);
    }

    fn serialize_as_felt252_vec(self) -> Vec<Felt252> {
        self.as_str().serialize_as_felt252_vec()
    }
}
