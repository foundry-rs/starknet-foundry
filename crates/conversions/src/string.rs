use crate::IntoConv;
use starknet_types_core::felt::{Felt as Felt252, FromStrError};

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

impl<T> IntoDecStr for T
where
    T: IntoConv<Felt252>,
{
    fn into_dec_string(self) -> String {
        self.into_().to_string()
    }
}

impl<T> IntoHexStr for T
where
    T: IntoConv<Felt252>,
{
    fn into_hex_string(self) -> String {
        self.into_().to_fixed_hex_string()
    }
}
