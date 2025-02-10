use crate::{
    byte_array::ByteArray,
    serde::serialize::SerializeToFeltVec,
    string::{TryFromDecStr, TryFromHexStr},
    FromConv, IntoConv,
};
use cairo_vm::utils::PRIME_STR;
use conversions::padded_felt::PaddedFelt;
use starknet::core::types::U256;
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

impl FromConv<U256> for Felt {
    fn from_(value: U256) -> Self {
        let felt_prime_high: u128 =
            u128::from_str_radix(&PRIME_STR[2..], 16).expect("Failed to parse prime");

        assert!(
            value.high() <= felt_prime_high,
            "U256 value is too large to fit into Felt"
        );

        assert!(
            !(value.high() == felt_prime_high && value.low() != 0),
            "U256 value is out of valid Felt range"
        );

        Felt::from(value.high()) * Felt::from(2).pow(128_u128) + Felt::from(value.low())
    }
}

// impl FromConv<U256> for Felt {
//     fn from_(value: U256) -> Self {
//         const FELT252_PRIME_HIGH: u128 =
//             u128::from_str_radix(&PRIME_STR[2..], 16).expect("Failed to parse prime");

//         assert!(
//             value.high() <= FELT252_PRIME_HIGH,
//             "U256 value is too large to fit into Felt"
//         );

//         assert!(
//             !(value.high() == FELT252_PRIME_HIGH && value.low() != 0),
//             "U256 value is out of valid Felt range"
//         );

//         Felt::from(value.high()) * Felt::from(2).pow(128_u128) + Felt::from(value.low())
//     }
// }

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
