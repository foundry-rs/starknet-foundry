pub mod class_hash;
pub mod contract_address;
pub mod dec_string;
pub mod felt252;
pub mod field_element;
pub mod nonce;
pub mod stark_felt;

pub trait FromConv<T> {
    fn from_(value: T) -> Self;
}

pub trait IntoConv<T>: Sized {
    fn into_(self) -> T;
}

impl<T, U> IntoConv<U> for T
where
    U: FromConv<T>,
{
    #[inline]
    fn into_(self: T) -> U {
        U::from_(self)
    }
}

#[macro_export]
macro_rules! from_thru_felt252 {
    ($from:ty, $to:ty) => {
        impl FromConv<$from> for $to {
            fn from_(value: $from) -> Self {
                Self::from_(Felt252::from_(value))
            }
        }
    };
}
