use crate::{from_thru_felt252, FromConv, IntoConv};
use starknet::core::types::FieldElement;
use starknet_api::core::{ClassHash, ContractAddress, EntryPointSelector, Nonce};
use starknet_types_core::felt::Felt as Felt252;

impl FromConv<Felt252> for EntryPointSelector {
    fn from_(value: Felt252) -> EntryPointSelector {
        EntryPointSelector(value.into_())
    }
}

from_thru_felt252!(FieldElement, EntryPointSelector);
from_thru_felt252!(ContractAddress, EntryPointSelector);
from_thru_felt252!(Nonce, EntryPointSelector);
from_thru_felt252!(ClassHash, EntryPointSelector);
