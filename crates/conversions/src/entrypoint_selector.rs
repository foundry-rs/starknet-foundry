use crate::{from_thru_felt252, try_from_str_thru_felt252, FromConv, IntoConv};
use cairo_felt::Felt252;
use starknet::core::types::FieldElement;
use starknet_api::core::{ClassHash, ContractAddress, EntryPointSelector, Nonce};
use starknet_api::hash::StarkFelt;

impl FromConv<Felt252> for EntryPointSelector {
    fn from_(value: Felt252) -> EntryPointSelector {
        EntryPointSelector(value.into_())
    }
}

from_thru_felt252!(FieldElement, EntryPointSelector);
from_thru_felt252!(StarkFelt, EntryPointSelector);
from_thru_felt252!(ContractAddress, EntryPointSelector);
from_thru_felt252!(Nonce, EntryPointSelector);
from_thru_felt252!(ClassHash, EntryPointSelector);

try_from_str_thru_felt252!(EntryPointSelector);
