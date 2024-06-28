use crate::{from_thru_felt252, FromConv};
use starknet::core::types::FieldElement;
use starknet_api::core::{ClassHash, ContractAddress, EntryPointSelector, Nonce};
use starknet_types_core::felt::Felt as Felt252;

impl FromConv<Felt252> for FieldElement {
    fn from_(value: Felt252) -> FieldElement {
        FieldElement::from_bytes_be(&value.to_bytes_be()).unwrap()
    }
}

from_thru_felt252!(ContractAddress, FieldElement);
from_thru_felt252!(ClassHash, FieldElement);
from_thru_felt252!(Nonce, FieldElement);
from_thru_felt252!(EntryPointSelector, FieldElement);
