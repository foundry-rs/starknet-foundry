use super::TryFromConv;
use starknet_types_core::felt::Felt;
use thiserror;

#[derive(Debug, thiserror::Error)]
pub enum PrimitiveConversionError {
    #[error("Felt overflow")]
    Overflow,
}

#[macro_export]
macro_rules! impl_try_from_felt {
    ($to:ty) => {
        impl TryFromConv<Felt> for $to {
            type Error = PrimitiveConversionError;
            fn try_from_(value: Felt) -> Result<$to, Self::Error> {
                if value.ge(&Felt::from(<$to>::MAX)) {
                    Err(PrimitiveConversionError::Overflow)
                } else {
                    Ok(<$to>::from_le_bytes(
                        value.to_bytes_le()[..size_of::<$to>()].try_into().unwrap(),
                    ))
                }
            }
        }
    };
}

impl_try_from_felt!(u64);
impl_try_from_felt!(u128);
