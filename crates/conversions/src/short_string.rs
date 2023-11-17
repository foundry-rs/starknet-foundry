use crate::{from_thru_felt252, FromConv};
use cairo_felt::Felt252;
use cairo_lang_runner::short_string::as_cairo_short_string;
use starknet::core::types::FieldElement;
use starknet_api::core::{ClassHash, ContractAddress, Nonce};
use starknet_api::hash::StarkFelt;

impl FromConv<Felt252> for String {
    fn from_(value: Felt252) -> String {
        as_cairo_short_string(&value).expect("Conversion to short string failed")
    }
}

from_thru_felt252!(FieldElement, String);
from_thru_felt252!(StarkFelt, String);
from_thru_felt252!(ClassHash, String);
from_thru_felt252!(ContractAddress, String);
from_thru_felt252!(Nonce, String);
