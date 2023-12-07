use crate::{from_thru_felt252, try_from_thru_felt252, FromConv, TryFromConv};
use cairo_felt::{Felt252, ParseFeltError};
use starknet::core::types::FieldElement;
use starknet_api::core::{ClassHash, ContractAddress, Nonce, PatriciaKey};
use starknet_api::hash::{StarkFelt, StarkHash};

impl FromConv<Felt252> for ContractAddress {
    fn from_(value: Felt252) -> ContractAddress {
        ContractAddress(PatriciaKey::try_from(StarkHash::from_(value)).unwrap())
    }
}

from_thru_felt252!(FieldElement, ContractAddress);
from_thru_felt252!(ClassHash, ContractAddress);
from_thru_felt252!(StarkFelt, ContractAddress);
from_thru_felt252!(Nonce, ContractAddress);

try_from_thru_felt252!(String, ContractAddress);
