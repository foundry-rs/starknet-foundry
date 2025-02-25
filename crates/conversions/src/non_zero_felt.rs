use crate::FromConv;
use starknet_types_core::felt::{Felt, NonZeroFelt};
use std::num::{NonZeroU64, NonZeroU128};

impl FromConv<NonZeroU64> for NonZeroFelt {
    fn from_(value: NonZeroU64) -> Self {
        NonZeroFelt::try_from(Felt::from(value.get())).unwrap_or_else(|_| {
            unreachable!(
                "NonZeroU64 is always greater than 0, so it should be convertible to NonZeroFelt"
            )
        })
    }
}

impl FromConv<NonZeroU128> for NonZeroFelt {
    fn from_(value: NonZeroU128) -> Self {
        NonZeroFelt::try_from(Felt::from(value.get())).unwrap_or_else(|_| {
            unreachable!(
                "NonZeroU128 is always greater than 0, so it should be convertible to NonZeroFelt"
            )
        })
    }
}
