pub mod class_hash;
pub mod contract_address;
pub mod felt252;
pub mod field_element;
pub mod nonce;
pub mod short_string;
pub mod stark_felt;

trait FromConv<T> {
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
