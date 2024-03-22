use crate::{from_thru_felt252, FromConv, IntoConv};
use cairo_felt::Felt252;
use starknet::core::types::FieldElement;
use starknet_api::core::{ClassHash, ContractAddress, EntryPointSelector, Nonce};
use starknet_api::hash::StarkFelt;

impl FromConv<Felt252> for ClassHash {
    fn from_(value: Felt252) -> ClassHash {
        ClassHash(value.into_())
    }
}

from_thru_felt252!(FieldElement, ClassHash);
from_thru_felt252!(StarkFelt, ClassHash);
from_thru_felt252!(ContractAddress, ClassHash);
from_thru_felt252!(Nonce, ClassHash);
from_thru_felt252!(EntryPointSelector, ClassHash);
