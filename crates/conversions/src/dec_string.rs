use crate::{from_thru_felt252, FromConv};
use cairo_felt::Felt252;
use starknet::core::types::FieldElement;
use starknet_api::core::{ClassHash, ContractAddress, Nonce};
use starknet_api::hash::StarkFelt;

impl FromConv<Felt252> for String {
    // Yields decimal string
    fn from_(value: Felt252) -> String {
        value.to_str_radix(10)
    }
}

from_thru_felt252!(FieldElement, String);
from_thru_felt252!(StarkFelt, String);
from_thru_felt252!(ClassHash, String);
from_thru_felt252!(ContractAddress, String);
from_thru_felt252!(Nonce, String);
