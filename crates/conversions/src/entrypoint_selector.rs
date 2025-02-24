use crate::{FromConv, IntoConv, from_thru_felt};
use starknet_api::core::{ClassHash, ContractAddress, EntryPointSelector, Nonce};
use starknet_types_core::felt::Felt;

impl FromConv<Felt> for EntryPointSelector {
    fn from_(value: Felt) -> EntryPointSelector {
        EntryPointSelector(value.into_())
    }
}

from_thru_felt!(ContractAddress, EntryPointSelector);
from_thru_felt!(Nonce, EntryPointSelector);
from_thru_felt!(ClassHash, EntryPointSelector);
