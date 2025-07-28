use crate::{
    FromConv, IntoConv,
    byte_array::ByteArray,
    serde::serialize::SerializeToFeltVec,
    string::{TryFromDecStr, TryFromHexStr},
};
use anyhow::{Context, Result, anyhow, bail};
use conversions::padded_felt::PaddedFelt;
use starknet_api::core::{ClassHash, ContractAddress, EntryPointSelector, Nonce};
use starknet_types_core::felt::Felt;
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
    fn try_from_dec_str(value: &str) -> Result<T> {
        if value.starts_with('-') {
            bail!("Value must not start with -")
        }

        Felt::from_dec_str(value)
            .map(T::from_)
            .with_context(|| anyhow!("Invalid value for string"))
    }
}

impl<T> TryFromHexStr for T
where
    T: FromConv<Felt>,
{
    fn try_from_hex_str(value: &str) -> Result<T> {
        if !value.starts_with("0x") {
            bail!("Value must start with 0x");
        }

        Felt::from_hex(value)
            .map(T::from_)
            .with_context(|| anyhow!("Invalid value for string"))
    }
}

pub trait FromShortString<T>: Sized {
    fn from_short_string(short_string: &str) -> Result<T>;
}

impl FromShortString<Felt> for Felt {
    fn from_short_string(short_string: &str) -> Result<Felt> {
        if short_string.len() <= 31 && short_string.is_ascii() {
            Ok(Felt::from_bytes_be_slice(short_string.as_bytes()))
        } else {
            bail!("Value must be ascii and less than 32 bytes long.")
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

pub trait TryInferFormat: Sized {
    /// Parses value from `hex string`, `dec string`, `quotted cairo shortstring `and `quotted cairo string`
    fn infer_format_and_parse(value: &str) -> Result<Vec<Self>>;
}

fn resolve(value: &str) -> String {
    value[1..value.len() - 1].replace("\\n", "\n")
}

impl TryInferFormat for Felt {
    fn infer_format_and_parse(value: &str) -> Result<Vec<Self>> {
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
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn short_string_happy_case() {
        let felt = Felt::from_hex("0x616263646566").unwrap();
        assert_eq!(felt.to_short_string().unwrap(), "abcdef");
    }

    #[test]
    fn short_string_31_characters() {
        let felt =
            Felt::from_hex("0x4142434445464748494a4b4c4d4e4f505152535455565758595a3132333435")
                .unwrap();
        assert_eq!(
            felt.to_short_string().unwrap(),
            "ABCDEFGHIJKLMNOPQRSTUVWXYZ12345"
        );
    }

    #[test]
    fn short_string_too_long() {
        let felt =
            Felt::from_hex("0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff")
                .unwrap();
        assert!(felt.to_short_string().is_err());
    }

    #[test]
    fn short_string_empty() {
        let felt = Felt::from_hex("0x0").unwrap();
        assert_eq!(felt.to_short_string().unwrap(), "");
    }

    #[test]
    fn short_string_with_whitespace() {
        let felt = Felt::from_hex("0x48656C6C6F20576F726C64").unwrap();
        assert_eq!(felt.to_short_string().unwrap(), "Hello World");
    }

    #[test]
    fn short_string_special_chars() {
        let felt = Felt::from_hex("0x4021233F2A2B5B5D").unwrap();
        assert_eq!(felt.to_short_string().unwrap(), "@!#?*+[]");
    }

    #[test]
    fn short_string_with_numbers() {
        let felt = Felt::from_hex("0x313233343536373839").unwrap();
        assert_eq!(felt.to_short_string().unwrap(), "123456789");
    }

    #[test]
    fn short_string_non_ascii() {
        let felt = Felt::from_hex("0x80").unwrap();
        assert!(felt.to_short_string().is_err());
    }

    #[test]
    fn short_string_null_byte() {
        let felt = Felt::from_hex("0x00616263").unwrap();
        assert_eq!(felt.to_short_string().unwrap(), "abc");
    }

    #[test]
    fn short_string_null_byte_middle() {
        let felt = Felt::from_hex("0x61006263").unwrap();
        assert!(felt.to_short_string().is_err());
    }

    #[test]
    fn short_string_null_byte_end() {
        let felt = Felt::from_hex("0x61626300").unwrap();
        assert_eq!(felt.to_short_string().unwrap(), "abc");
    }
}
