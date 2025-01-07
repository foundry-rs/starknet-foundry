use crate::TryFromConv;
use starknet_types_core::felt::{Felt, NonZeroFelt};
use std::num::{NonZero, NonZeroU64};

impl TryFromConv<NonZeroFelt> for NonZeroU64 {
    type Error = String;
    fn try_from_(value: NonZeroFelt) -> Result<Self, Self::Error> {
        let value: u64 = Felt::from(value)
            .try_into()
            .map_err(|_| "felt was too large to fit in u64")?;
        Ok(NonZero::new(value)
            .unwrap_or_else(|| unreachable!("non zero felt is always greater than 0")))
    }
}
