use crate::{from_thru_felt252, FromConv, IntoConv};
use starknet_api::core::{ClassHash, ContractAddress, EntryPointSelector, Nonce};
use starknet_types_core::felt::Felt as Felt252;

impl FromConv<Felt252> for ClassHash {
    fn from_(value: Felt252) -> ClassHash {
        ClassHash(value.into_())
    }
}

from_thru_felt252!(ContractAddress, ClassHash);
from_thru_felt252!(Nonce, ClassHash);
from_thru_felt252!(EntryPointSelector, ClassHash);
