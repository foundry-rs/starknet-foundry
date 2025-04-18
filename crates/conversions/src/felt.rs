use crate::{
    FromConv, IntoConv,
    byte_array::ByteArray,
    serde::serialize::SerializeToFeltVec,
    string::{TryFromDecStr, TryFromHexStr},
};
use conversions::padded_felt::PaddedFelt;
use starknet_api::core::{ClassHash, ContractAddress, EntryPointSelector, Nonce};
use starknet_types_core::felt::{Felt, FromStrError};
use std::vec;

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

#[derive(Debug)]
pub struct ToStrErr;

pub trait ToShortString<T>: Sized {
    fn to_short_string(&self) -> Result<String, ToStrErr>;
}

impl ToShortString<Felt> for Felt {
    fn to_short_string(&self) -> Result<String, ToStrErr> {
        let mut as_string = String::default();
        let mut is_end = false;
        for byte in self.to_biguint().to_bytes_be() {
            if byte == 0 {
                is_end = true;
            } else if is_end {
                return Err(ToStrErr);
            } else if byte.is_ascii_graphic() || byte.is_ascii_whitespace() {
                as_string.push(byte as char);
            } else {
                return Err(ToStrErr);
            }
        }
        Ok(as_string)
    }
}

pub trait ToFixedLengthShortString<T>: Sized {
    fn to_fixed_length_short_string(&self, length: usize) -> Result<String, ToStrErr>;
}

impl ToFixedLengthShortString<Felt> for Felt {
    fn to_fixed_length_short_string(&self, length: usize) -> Result<String, ToStrErr> {
        if length == 0 {
            return if *self == Felt::ZERO {
                Ok(String::new())
            } else {
                Err(ToStrErr)
            };
        }
        if length > 31 {
            // A short string can't be longer than 31 bytes.
            return Err(ToStrErr);
        }

        // We pass through biguint as felt252.to_bytes_be() does not trim leading zeros.
        let bytes = self.to_biguint().to_bytes_be();
        let bytes_len = bytes.len();
        if bytes_len > length {
            // `value` has more bytes than expected.
            return Err(ToStrErr);
        }

        let mut as_string = String::new();
        for byte in bytes {
            if byte == 0 {
                as_string.push_str(r"\0");
            } else if byte.is_ascii_graphic() || byte.is_ascii_whitespace() {
                as_string.push(byte as char);
            } else {
                as_string.push_str(format!(r"\x{byte:02x}").as_str());
            }
        }

        // `to_bytes_be` misses starting nulls. Prepend them as needed.
        let missing_nulls = length - bytes_len;
        as_string.insert_str(0, &r"\0".repeat(missing_nulls));

        Ok(as_string)
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
