use crate::{FromConv, IntoConv};
use cairo_felt::Felt252;
use starknet_api::core::Nonce;

impl FromConv<Felt252> for Nonce {
    fn from_(value: Felt252) -> Nonce {
        Nonce(value.into_())
    }
}
