use crate::TryFromConv;
use starknet_types_core::felt::{Felt, NonZeroFelt};
use std::num::{NonZero, NonZeroU128};

impl TryFromConv<NonZeroFelt> for NonZeroU128 {
    type Error = String;
    fn try_from_(value: NonZeroFelt) -> Result<Self, Self::Error> {
        let value: u128 = Felt::from(value)
            .try_into()
            .map_err(|_| "felt was too large to fit in u128")?;
        Ok(NonZero::new(value)
            .unwrap_or_else(|| unreachable!("non zero felt is always greater than 0")))
    }
}
