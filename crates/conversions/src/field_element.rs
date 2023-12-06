use crate::{from_thru_felt252, try_from_thru_felt252, FromConv, TryFromConv};
use cairo_felt::{Felt252, ParseFeltError};
use starknet::core::types::FieldElement;
use starknet_api::core::{ClassHash, ContractAddress, Nonce};
use starknet_api::hash::StarkFelt;

impl FromConv<Felt252> for FieldElement {
    fn from_(value: Felt252) -> FieldElement {
        FieldElement::from_bytes_be(&value.to_be_bytes()).unwrap()
    }
}

from_thru_felt252!(ContractAddress, FieldElement);
from_thru_felt252!(StarkFelt, FieldElement);
from_thru_felt252!(ClassHash, FieldElement);
from_thru_felt252!(Nonce, FieldElement);

try_from_thru_felt252!(String, FieldElement);
