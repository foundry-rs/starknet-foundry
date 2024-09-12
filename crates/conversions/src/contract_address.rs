use crate::{from_thru_felt252, FromConv};
use starknet_api::core::{ClassHash, ContractAddress, EntryPointSelector, Nonce, PatriciaKey};
use starknet_api::hash::StarkHash;
use starknet_types_core::felt::Felt as Felt252;

impl FromConv<Felt252> for ContractAddress {
    fn from_(value: Felt252) -> ContractAddress {
        ContractAddress(PatriciaKey::try_from(StarkHash::from_(value)).unwrap())
    }
}

from_thru_felt252!(ClassHash, ContractAddress);
from_thru_felt252!(Nonce, ContractAddress);
from_thru_felt252!(EntryPointSelector, ContractAddress);
