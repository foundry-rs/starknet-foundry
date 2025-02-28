use crate::{from_thru_felt, FromConv};
use conversions::padded_felt::PaddedFelt;
use starknet_api::core::{ClassHash, ContractAddress, EntryPointSelector, Nonce, PatriciaKey};
use starknet_api::hash::StarkHash;
use starknet_types_core::felt::Felt;

impl FromConv<Felt> for ContractAddress {
    fn from_(value: Felt) -> ContractAddress {
        ContractAddress(PatriciaKey::try_from(StarkHash::from_(value)).unwrap())
    }
}

from_thru_felt!(ClassHash, ContractAddress);
from_thru_felt!(Nonce, ContractAddress);
from_thru_felt!(EntryPointSelector, ContractAddress);
from_thru_felt!(PaddedFelt, ContractAddress);
