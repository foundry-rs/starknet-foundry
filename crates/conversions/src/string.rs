use crate::IntoConv;
use starknet_types_core::felt::{Felt, FromStrError};

pub trait TryFromDecStr {
    fn try_from_dec_str(str: &str) -> Result<Self, FromStrError>
    where
        Self: Sized;
}

pub trait TryFromHexStr {
    fn try_from_hex_str(str: &str) -> Result<Self, FromStrError>
    where
        Self: Sized;
}
pub trait IntoDecStr {
    fn into_dec_string(self) -> String;
}

pub trait IntoHexStr {
    fn into_hex_string(self) -> String;
}

pub trait IntoPaddedHexStr {
    fn into_padded_hex_str(self) -> String;
}

impl<T> IntoDecStr for T
where
    T: IntoConv<Felt>,
{
    fn into_dec_string(self) -> String {
        self.into_().to_string()
    }
}

impl<T> IntoHexStr for T
where
    T: IntoConv<Felt>,
{
    fn into_hex_string(self) -> String {
        self.into_().to_hex_string()
    }
}

impl<T> IntoPaddedHexStr for T
where
    T: IntoConv<Felt>,
{
    fn into_padded_hex_str(self) -> String {
        self.into_().to_fixed_hex_string()
    }
}
