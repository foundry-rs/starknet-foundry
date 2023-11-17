use crate::{FromConv, IntoConv};
use cairo_felt::Felt252;
use starknet_api::core::ClassHash;

impl FromConv<Felt252> for ClassHash {
    fn from_(value: Felt252) -> ClassHash {
        ClassHash(value.into_())
    }
}
