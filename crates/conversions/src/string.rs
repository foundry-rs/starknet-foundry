use crate::IntoConv;
use cairo_felt::Felt252;
use thiserror::Error;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Error)]
pub struct ParseFeltError;

impl std::fmt::Display for ParseFeltError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

pub trait TryFromDecStr {
    fn try_from_dec_str(str: &str) -> Result<Self, ParseFeltError>
    where
        Self: Sized;
}

pub trait TryFromHexStr {
    fn try_from_hex_str(str: &str) -> Result<Self, ParseFeltError>
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
        self.into_().to_str_radix(10)
    }
}

impl<T> IntoHexStr for T
where
    T: IntoConv<Felt252>,
{
    fn into_hex_string(self) -> String {
        let mut result = self.into_().to_str_radix(16);

        result.insert_str(0, "0x");

        result
    }
}
