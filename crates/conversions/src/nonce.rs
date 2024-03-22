use crate::{from_thru_felt252, FromConv, IntoConv};
use cairo_felt::Felt252;
use starknet::core::types::FieldElement;
use starknet_api::core::{ClassHash, ContractAddress, EntryPointSelector, Nonce};
use starknet_api::hash::StarkFelt;

impl FromConv<Felt252> for Nonce {
    fn from_(value: Felt252) -> Nonce {
        Nonce(value.into_())
    }
}

from_thru_felt252!(FieldElement, Nonce);
from_thru_felt252!(StarkFelt, Nonce);
from_thru_felt252!(ClassHash, Nonce);
from_thru_felt252!(ContractAddress, Nonce);
from_thru_felt252!(EntryPointSelector, Nonce);
