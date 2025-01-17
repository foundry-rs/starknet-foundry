use std::convert::Infallible;

pub mod byte_array;
pub mod class_hash;
pub mod contract_address;
pub mod entrypoint_selector;
pub mod eth_address;
pub mod felt;
pub mod non_zero_felt;
pub mod non_zero_u128;
pub mod non_zero_u64;
pub mod nonce;
pub mod padded_felt;
pub mod primitive;
pub mod serde;
pub mod string;

extern crate self as conversions;

pub trait FromConv<T>: Sized {
    fn from_(value: T) -> Self;
}

impl<T> FromConv<T> for T {
    fn from_(value: T) -> Self {
        value
    }
}

pub trait IntoConv<T>: Sized {
    fn into_(self) -> T;
}

// FromConv implies IntoConv
impl<T, U> IntoConv<U> for T
where
    U: FromConv<T>,
{
    #[inline]
    fn into_(self: T) -> U {
        U::from_(self)
    }
}

pub trait TryFromConv<T>: Sized {
    type Error;

    fn try_from_(value: T) -> Result<Self, Self::Error>;
}

pub trait TryIntoConv<T>: Sized {
    type Error;

    fn try_into_(self) -> Result<T, Self::Error>;
}

// TryFromConv implies TryIntoConv
impl<T, U> TryIntoConv<U> for T
where
    U: TryFromConv<T>,
{
    type Error = U::Error;

    #[inline]
    fn try_into_(self) -> Result<U, U::Error> {
        U::try_from_(self)
    }
}

// Infallible conversions are semantically equivalent to fallible conversions
// with an uninhabited error type.
impl<T, U> TryFromConv<U> for T
where
    U: IntoConv<T>,
{
    type Error = Infallible;

    #[inline]
    fn try_from_(value: U) -> Result<Self, Self::Error> {
        Ok(U::into_(value))
    }
}

#[macro_export]
macro_rules! from_thru_felt {
    ($from:ty, $to:ty) => {
        impl FromConv<$from> for $to {
            fn from_(value: $from) -> Self {
                Self::from_(Felt::from_(value))
            }
        }
    };
}
