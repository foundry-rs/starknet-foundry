use crate::{
    byte_array::ByteArray,
    serde::serialize::SerializeToFeltVec,
    string::{TryFromDecStr, TryFromHexStr},
    FromConv, IntoConv, TryFromConv,
};
use conversions::padded_felt::PaddedFelt;
use starknet_api::core::{ClassHash, ContractAddress, EntryPointSelector, Nonce};
use starknet_types_core::felt::{Felt, FromStrError};
use std::{
    num::{NonZeroU128, NonZeroU64},
    vec,
};

impl FromConv<ClassHash> for Felt {
    fn from_(value: ClassHash) -> Felt {
        value.0.into_()
    }
}

impl FromConv<ContractAddress> for Felt {
    fn from_(value: ContractAddress) -> Felt {
        (*value.0.key()).into_()
    }
}

impl FromConv<Nonce> for Felt {
    fn from_(value: Nonce) -> Felt {
        value.0.into_()
    }
}

impl FromConv<EntryPointSelector> for Felt {
    fn from_(value: EntryPointSelector) -> Felt {
        value.0.into_()
    }
}

impl FromConv<PaddedFelt> for Felt {
    fn from_(value: PaddedFelt) -> Felt {
        value.0.into_()
    }
}

impl TryFromConv<Felt> for NonZeroU64 {
    type Error = String;
    fn try_from_(value: Felt) -> Result<Self, Self::Error> {
        if value == Felt::ZERO {
            Err("value should be greater than 0".to_string())
        } else {
            let value: u64 = value.try_into().expect("failed to convert Felt to u64");
            Ok(NonZeroU64::new(value).unwrap())
        }
    }
}

impl TryFromConv<Felt> for NonZeroU128 {
    type Error = String;
    fn try_from_(value: Felt) -> Result<Self, Self::Error> {
        if value == Felt::ZERO {
            Err("value should be greater than 0".to_string())
        } else {
            let value: u128 = value.try_into().expect("failed to convert Felt to u128");
            Ok(NonZeroU128::new(value).unwrap())
        }
    }
}

impl FromConv<NonZeroU64> for Felt {
    fn from_(value: NonZeroU64) -> Felt {
        Felt::from(value.get())
    }
}

impl FromConv<NonZeroU128> for Felt {
    fn from_(value: NonZeroU128) -> Felt {
        Felt::from(value.get())
    }
}

impl<T> TryFromDecStr for T
where
    T: FromConv<Felt>,
{
    fn try_from_dec_str(value: &str) -> Result<T, FromStrError> {
        if value.starts_with('-') {
            return Err(FromStrError);
        }

        Felt::from_dec_str(value).map(T::from_)
    }
}

impl<T> TryFromHexStr for T
where
    T: FromConv<Felt>,
{
    fn try_from_hex_str(value: &str) -> Result<T, FromStrError> {
        if !value.starts_with("0x") {
            return Err(FromStrError);
        }

        Felt::from_hex(value).map(T::from_)
    }
}

pub trait FromShortString<T>: Sized {
    fn from_short_string(short_string: &str) -> Result<T, FromStrError>;
}

impl FromShortString<Felt> for Felt {
    fn from_short_string(short_string: &str) -> Result<Felt, FromStrError> {
        if short_string.len() <= 31 && short_string.is_ascii() {
            Ok(Felt::from_bytes_be_slice(short_string.as_bytes()))
        } else {
            Err(FromStrError)
        }
    }
}

pub trait TryInferFormat: Sized {
    /// Parses value from `hex string`, `dec string`, `quotted cairo shortstring `and `quotted cairo string`
    fn infer_format_and_parse(value: &str) -> Result<Vec<Self>, FromStrError>;
}

fn resolve(value: &str) -> String {
    value[1..value.len() - 1].replace("\\n", "\n")
}

impl TryInferFormat for Felt {
    fn infer_format_and_parse(value: &str) -> Result<Vec<Self>, FromStrError> {
        if value.starts_with('\'') && value.ends_with('\'') {
            let value = resolve(value).replace("\\'", "'");

            Felt::from_short_string(&value).map(|felt| vec![felt])
        } else if value.starts_with('"') && value.ends_with('"') {
            let value = resolve(value).replace("\\\"", "\"");

            Ok(ByteArray::from(value.as_str()).serialize_to_vec())
        } else {
            Felt::try_from_hex_str(value)
                .or_else(|_| Felt::try_from_dec_str(value))
                .map(|felt| vec![felt])
        }
    }
}
