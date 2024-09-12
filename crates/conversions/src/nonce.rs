use crate::{from_thru_felt252, FromConv, IntoConv};
use starknet_api::core::{ClassHash, ContractAddress, EntryPointSelector, Nonce};
use starknet_types_core::felt::Felt as Felt252;

impl FromConv<Felt252> for Nonce {
    fn from_(value: Felt252) -> Nonce {
        Nonce(value.into_())
    }
}

from_thru_felt252!(ClassHash, Nonce);
from_thru_felt252!(ContractAddress, Nonce);
from_thru_felt252!(EntryPointSelector, Nonce);
